#include "renderer_vulkan.hpp"

#include "../../app/log.hpp"
#include "../../assets/asset_fs.hpp"

#ifdef __ANDROID__
#include <android/native_window.h>
#else
#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#ifndef NOMINMAX
#define NOMINMAX
#endif
#include <windows.h>
#endif

#include <algorithm>
#include <cstring>
#include <fstream>
#include <vector>

namespace {

VKAPI_ATTR VkBool32 VKAPI_CALL debug_callback(
    VkDebugUtilsMessageSeverityFlagBitsEXT severity,
    VkDebugUtilsMessageTypeFlagsEXT,
    const VkDebugUtilsMessengerCallbackDataEXT* data,
    void*) {
    if (severity >= VK_DEBUG_UTILS_MESSAGE_SEVERITY_WARNING_BIT_EXT) {
        vs_log("Vulkan: %s", data->pMessage ? data->pMessage : "(null)");
    }
    return VK_FALSE;
}

void transition_image(
    VkCommandBuffer cmd,
    VkImage image,
    VkImageLayout old_layout,
    VkImageLayout new_layout,
    VkAccessFlags src_access,
    VkAccessFlags dst_access,
    VkPipelineStageFlags src_stage,
    VkPipelineStageFlags dst_stage) {
    VkImageMemoryBarrier barrier{};
    barrier.sType = VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER;
    barrier.oldLayout = old_layout;
    barrier.newLayout = new_layout;
    barrier.srcQueueFamilyIndex = VK_QUEUE_FAMILY_IGNORED;
    barrier.dstQueueFamilyIndex = VK_QUEUE_FAMILY_IGNORED;
    barrier.image = image;
    barrier.subresourceRange.aspectMask = VK_IMAGE_ASPECT_COLOR_BIT;
    barrier.subresourceRange.levelCount = 1;
    barrier.subresourceRange.layerCount = 1;
    barrier.srcAccessMask = src_access;
    barrier.dstAccessMask = dst_access;
    vkCmdPipelineBarrier(cmd, src_stage, dst_stage, 0, 0, nullptr, 0, nullptr, 1, &barrier);
}

} // namespace

Renderer* create_vulkan_renderer() {
    return new RendererVulkan();
}

bool RendererVulkan::init(const RendererCreateInfo& info) {
    validation_ = info.enable_validation;
    shader_dir_ = info.shader_dir ? info.shader_dir : "shaders";
    module_handle_ = info.module_handle;

    if (volkInitialize() != VK_SUCCESS) {
        vs_log("Failed to initialize volk / Vulkan loader");
        return false;
    }
    if (!create_instance(validation_)) {
        return false;
    }
    volkLoadInstance(instance_);
    if (!create_surface(info.window_handle, info.module_handle)) {
        return false;
    }
    if (!pick_device() || !create_device()) {
        return false;
    }
    volkLoadDevice(device_);
    if (!create_command_pool() || !create_frame_sync()) {
        return false;
    }
    if (!create_render_pass() || !create_descriptors() || !create_pipeline()) {
        return false;
    }
    if (!create_geometry_buffers()) {
        return false;
    }
    if (!create_swapchain(info.width, info.height)) {
        return false;
    }
#ifdef VS_GPU_TIMESTAMPS
    {
        VkPhysicalDeviceProperties props{};
        vkGetPhysicalDeviceProperties(physical_, &props);
        timestamp_period_ns_ = props.limits.timestampPeriod;
        VkQueryPoolCreateInfo qci{};
        qci.sType = VK_STRUCTURE_TYPE_QUERY_POOL_CREATE_INFO;
        qci.queryType = VK_QUERY_TYPE_TIMESTAMP;
        qci.queryCount = 2;
        if (vkCreateQueryPool(device_, &qci, nullptr, &timestamp_pool_) != VK_SUCCESS) {
            timestamp_pool_ = VK_NULL_HANDLE;
            vs_log("GPU timestamps unavailable (query pool)");
        } else {
            vs_log("GPU timestamps ON (period=%.3fns)", timestamp_period_ns_);
        }
    }
#endif
    camera_.set_viewport(static_cast<float>(info.width), static_cast<float>(info.height));
    mat3_to_mat4(camera_.view_projection(), view_proj_);
    vs_log("Vulkan renderer initialized (%dx%d) shaders=%s", info.width, info.height, shader_dir_.c_str());
    return true;
}

void RendererVulkan::shutdown() {
    if (device_) {
        vkDeviceWaitIdle(device_);
    }
    for (GpuTexture& tex : textures_) {
        destroy_gpu_texture(tex);
    }
    textures_.clear();

#ifdef VS_GPU_TIMESTAMPS
    if (timestamp_pool_) {
        vkDestroyQueryPool(device_, timestamp_pool_, nullptr);
        timestamp_pool_ = VK_NULL_HANDLE;
    }
#endif

    destroy_swapchain();

    if (vertex_mapped_) {
        vkUnmapMemory(device_, vertex_memory_);
        vertex_mapped_ = nullptr;
    }
    if (vertex_buffer_) {
        vkDestroyBuffer(device_, vertex_buffer_, nullptr);
        vertex_buffer_ = VK_NULL_HANDLE;
    }
    if (vertex_memory_) {
        vkFreeMemory(device_, vertex_memory_, nullptr);
        vertex_memory_ = VK_NULL_HANDLE;
    }
    if (index_buffer_) {
        vkDestroyBuffer(device_, index_buffer_, nullptr);
        index_buffer_ = VK_NULL_HANDLE;
    }
    if (index_memory_) {
        vkFreeMemory(device_, index_memory_, nullptr);
        index_memory_ = VK_NULL_HANDLE;
    }
    if (pipeline_) {
        vkDestroyPipeline(device_, pipeline_, nullptr);
        pipeline_ = VK_NULL_HANDLE;
    }
    if (pipeline_layout_) {
        vkDestroyPipelineLayout(device_, pipeline_layout_, nullptr);
        pipeline_layout_ = VK_NULL_HANDLE;
    }
    if (descriptor_pool_) {
        vkDestroyDescriptorPool(device_, descriptor_pool_, nullptr);
        descriptor_pool_ = VK_NULL_HANDLE;
    }
    if (descriptor_set_layout_) {
        vkDestroyDescriptorSetLayout(device_, descriptor_set_layout_, nullptr);
        descriptor_set_layout_ = VK_NULL_HANDLE;
    }
    if (sampler_) {
        vkDestroySampler(device_, sampler_, nullptr);
        sampler_ = VK_NULL_HANDLE;
    }
    if (render_pass_) {
        vkDestroyRenderPass(device_, render_pass_, nullptr);
        render_pass_ = VK_NULL_HANDLE;
    }
    if (in_flight_) {
        vkDestroyFence(device_, in_flight_, nullptr);
        in_flight_ = VK_NULL_HANDLE;
    }
    if (image_available_) {
        vkDestroySemaphore(device_, image_available_, nullptr);
        image_available_ = VK_NULL_HANDLE;
    }
    if (command_pool_) {
        vkDestroyCommandPool(device_, command_pool_, nullptr);
        command_pool_ = VK_NULL_HANDLE;
    }
    if (device_) {
        vkDestroyDevice(device_, nullptr);
        device_ = VK_NULL_HANDLE;
    }
    if (surface_) {
        vkDestroySurfaceKHR(instance_, surface_, nullptr);
        surface_ = VK_NULL_HANDLE;
    }
    if (debug_messenger_) {
        auto destroy = reinterpret_cast<PFN_vkDestroyDebugUtilsMessengerEXT>(
            vkGetInstanceProcAddr(instance_, "vkDestroyDebugUtilsMessengerEXT"));
        if (destroy) {
            destroy(instance_, debug_messenger_, nullptr);
        }
        debug_messenger_ = VK_NULL_HANDLE;
    }
    if (instance_) {
        vkDestroyInstance(instance_, nullptr);
        instance_ = VK_NULL_HANDLE;
    }
}

bool RendererVulkan::resize(int width, int height) {
    if (width <= 0 || height <= 0 || !device_) {
        return true;
    }
    vkDeviceWaitIdle(device_);
    destroy_swapchain();
    camera_.set_viewport(static_cast<float>(width), static_cast<float>(height));
    return create_swapchain(width, height);
}

void RendererVulkan::mat3_to_mat4(const Mat3& m, float out16[16]) const {
    // Column-major mat4 from 2D affine Mat3 (translation in third column → mat4 col3).
    out16[0] = m.m[0];
    out16[1] = m.m[1];
    out16[2] = 0.0f;
    out16[3] = 0.0f;
    out16[4] = m.m[3];
    out16[5] = m.m[4];
    out16[6] = 0.0f;
    out16[7] = 0.0f;
    out16[8] = 0.0f;
    out16[9] = 0.0f;
    out16[10] = 1.0f;
    out16[11] = 0.0f;
    out16[12] = m.m[6];
    out16[13] = m.m[7];
    out16[14] = 0.0f;
    out16[15] = 1.0f;
}

uint32_t RendererVulkan::find_memory_type(uint32_t type_bits, VkMemoryPropertyFlags flags) const {
    VkPhysicalDeviceMemoryProperties props{};
    vkGetPhysicalDeviceMemoryProperties(physical_, &props);
    for (uint32_t i = 0; i < props.memoryTypeCount; ++i) {
        if ((type_bits & (1u << i)) && (props.memoryTypes[i].propertyFlags & flags) == flags) {
            return i;
        }
    }
    return UINT32_MAX;
}

bool RendererVulkan::create_buffer(
    VkDeviceSize size,
    VkBufferUsageFlags usage,
    VkMemoryPropertyFlags mem_flags,
    VkBuffer* buffer,
    VkDeviceMemory* memory) {
    VkBufferCreateInfo bi{};
    bi.sType = VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO;
    bi.size = size;
    bi.usage = usage;
    bi.sharingMode = VK_SHARING_MODE_EXCLUSIVE;
    if (vkCreateBuffer(device_, &bi, nullptr, buffer) != VK_SUCCESS) {
        return false;
    }
    VkMemoryRequirements req{};
    vkGetBufferMemoryRequirements(device_, *buffer, &req);
    VkMemoryAllocateInfo ai{};
    ai.sType = VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO;
    ai.allocationSize = req.size;
    ai.memoryTypeIndex = find_memory_type(req.memoryTypeBits, mem_flags);
    if (ai.memoryTypeIndex == UINT32_MAX) {
        return false;
    }
    if (vkAllocateMemory(device_, &ai, nullptr, memory) != VK_SUCCESS) {
        return false;
    }
    vkBindBufferMemory(device_, *buffer, *memory, 0);
    return true;
}

bool RendererVulkan::load_shader_module(const char* filename, VkShaderModule* out) {
    const std::string path = shader_dir_ + "/" + filename;
    std::vector<char> buffer;
    if (AssetFs* fs = vs_asset_fs()) {
        std::vector<uint8_t> bytes;
        // Prefer packaged "shaders/foo.spv" then absolute build path.
        if (fs->read_bytes(("shaders/" + std::string(filename)).c_str(), bytes) ||
            fs->read_bytes(path.c_str(), bytes)) {
            buffer.assign(bytes.begin(), bytes.end());
        }
    }
    if (buffer.empty()) {
        std::ifstream file(path, std::ios::binary | std::ios::ate);
        if (!file) {
            vs_log("Missing shader: %s", path.c_str());
            return false;
        }
        const std::streamsize size = file.tellg();
        file.seekg(0, std::ios::beg);
        buffer.resize(static_cast<size_t>(size));
        if (!file.read(buffer.data(), size)) {
            return false;
        }
    }
    VkShaderModuleCreateInfo ci{};
    ci.sType = VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO;
    ci.codeSize = buffer.size();
    ci.pCode = reinterpret_cast<const uint32_t*>(buffer.data());
    if (vkCreateShaderModule(device_, &ci, nullptr, out) != VK_SUCCESS) {
        vs_log("vkCreateShaderModule failed for %s", path.c_str());
        return false;
    }
    return true;
}

bool RendererVulkan::create_render_pass() {
    VkAttachmentDescription color{};
    color.format = VK_FORMAT_B8G8R8A8_UNORM; // updated when swapchain created; recreate on format change
    color.samples = VK_SAMPLE_COUNT_1_BIT;
    color.loadOp = VK_ATTACHMENT_LOAD_OP_CLEAR;
    color.storeOp = VK_ATTACHMENT_STORE_OP_STORE;
    color.stencilLoadOp = VK_ATTACHMENT_LOAD_OP_DONT_CARE;
    color.stencilStoreOp = VK_ATTACHMENT_STORE_OP_DONT_CARE;
    color.initialLayout = VK_IMAGE_LAYOUT_UNDEFINED;
    color.finalLayout = VK_IMAGE_LAYOUT_PRESENT_SRC_KHR;

    VkAttachmentReference color_ref{};
    color_ref.attachment = 0;
    color_ref.layout = VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL;

    VkSubpassDescription subpass{};
    subpass.pipelineBindPoint = VK_PIPELINE_BIND_POINT_GRAPHICS;
    subpass.colorAttachmentCount = 1;
    subpass.pColorAttachments = &color_ref;

    VkSubpassDependency dep{};
    dep.srcSubpass = VK_SUBPASS_EXTERNAL;
    dep.dstSubpass = 0;
    dep.srcStageMask = VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT;
    dep.dstStageMask = VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT;
    dep.dstAccessMask = VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT;

    VkRenderPassCreateInfo ci{};
    ci.sType = VK_STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO;
    ci.attachmentCount = 1;
    ci.pAttachments = &color;
    ci.subpassCount = 1;
    ci.pSubpasses = &subpass;
    ci.dependencyCount = 1;
    ci.pDependencies = &dep;
    return vkCreateRenderPass(device_, &ci, nullptr, &render_pass_) == VK_SUCCESS;
}

bool RendererVulkan::create_descriptors() {
    VkSamplerCreateInfo sci{};
    sci.sType = VK_STRUCTURE_TYPE_SAMPLER_CREATE_INFO;
    sci.magFilter = VK_FILTER_LINEAR;
    sci.minFilter = VK_FILTER_LINEAR;
    sci.addressModeU = VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE;
    sci.addressModeV = VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE;
    sci.addressModeW = VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE;
    sci.maxLod = 0.0f;
    if (vkCreateSampler(device_, &sci, nullptr, &sampler_) != VK_SUCCESS) {
        return false;
    }

    VkDescriptorSetLayoutBinding binding{};
    binding.binding = 0;
    binding.descriptorType = VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER;
    binding.descriptorCount = 1;
    binding.stageFlags = VK_SHADER_STAGE_FRAGMENT_BIT;

    VkDescriptorSetLayoutCreateInfo lci{};
    lci.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO;
    lci.bindingCount = 1;
    lci.pBindings = &binding;
    if (vkCreateDescriptorSetLayout(device_, &lci, nullptr, &descriptor_set_layout_) != VK_SUCCESS) {
        return false;
    }

    VkDescriptorPoolSize pool_size{};
    pool_size.type = VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER;
    pool_size.descriptorCount = 64;
    VkDescriptorPoolCreateInfo pci{};
    pci.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO;
    pci.poolSizeCount = 1;
    pci.pPoolSizes = &pool_size;
    pci.maxSets = 64;
    pci.flags = VK_DESCRIPTOR_POOL_CREATE_FREE_DESCRIPTOR_SET_BIT;
    return vkCreateDescriptorPool(device_, &pci, nullptr, &descriptor_pool_) == VK_SUCCESS;
}

bool RendererVulkan::create_pipeline() {
    VkShaderModule vert{};
    VkShaderModule frag{};
    if (!load_shader_module("sprite.vert.spv", &vert) || !load_shader_module("sprite.frag.spv", &frag)) {
        return false;
    }

    VkPipelineShaderStageCreateInfo stages[2]{};
    stages[0].sType = VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO;
    stages[0].stage = VK_SHADER_STAGE_VERTEX_BIT;
    stages[0].module = vert;
    stages[0].pName = "main";
    stages[1].sType = VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO;
    stages[1].stage = VK_SHADER_STAGE_FRAGMENT_BIT;
    stages[1].module = frag;
    stages[1].pName = "main";

    VkVertexInputBindingDescription binding{};
    binding.binding = 0;
    binding.stride = sizeof(SpriteVertex);
    binding.inputRate = VK_VERTEX_INPUT_RATE_VERTEX;

    VkVertexInputAttributeDescription attrs[3]{};
    attrs[0].location = 0;
    attrs[0].binding = 0;
    attrs[0].format = VK_FORMAT_R32G32_SFLOAT;
    attrs[0].offset = offsetof(SpriteVertex, x);
    attrs[1].location = 1;
    attrs[1].binding = 0;
    attrs[1].format = VK_FORMAT_R32G32_SFLOAT;
    attrs[1].offset = offsetof(SpriteVertex, u);
    attrs[2].location = 2;
    attrs[2].binding = 0;
    attrs[2].format = VK_FORMAT_R32G32B32A32_SFLOAT;
    attrs[2].offset = offsetof(SpriteVertex, r);

    VkPipelineVertexInputStateCreateInfo vi{};
    vi.sType = VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO;
    vi.vertexBindingDescriptionCount = 1;
    vi.pVertexBindingDescriptions = &binding;
    vi.vertexAttributeDescriptionCount = 3;
    vi.pVertexAttributeDescriptions = attrs;

    VkPipelineInputAssemblyStateCreateInfo ia{};
    ia.sType = VK_STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO;
    ia.topology = VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST;

    VkPipelineViewportStateCreateInfo vp{};
    vp.sType = VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_STATE_CREATE_INFO;
    vp.viewportCount = 1;
    vp.scissorCount = 1;

    VkPipelineRasterizationStateCreateInfo rs{};
    rs.sType = VK_STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO;
    rs.polygonMode = VK_POLYGON_MODE_FILL;
    rs.cullMode = VK_CULL_MODE_NONE;
    rs.frontFace = VK_FRONT_FACE_COUNTER_CLOCKWISE;
    rs.lineWidth = 1.0f;

    VkPipelineMultisampleStateCreateInfo ms{};
    ms.sType = VK_STRUCTURE_TYPE_PIPELINE_MULTISAMPLE_STATE_CREATE_INFO;
    ms.rasterizationSamples = VK_SAMPLE_COUNT_1_BIT;

    VkPipelineColorBlendAttachmentState blend_att{};
    blend_att.blendEnable = VK_TRUE;
    blend_att.srcColorBlendFactor = VK_BLEND_FACTOR_SRC_ALPHA;
    blend_att.dstColorBlendFactor = VK_BLEND_FACTOR_ONE_MINUS_SRC_ALPHA;
    blend_att.colorBlendOp = VK_BLEND_OP_ADD;
    blend_att.srcAlphaBlendFactor = VK_BLEND_FACTOR_ONE;
    blend_att.dstAlphaBlendFactor = VK_BLEND_FACTOR_ONE_MINUS_SRC_ALPHA;
    blend_att.alphaBlendOp = VK_BLEND_OP_ADD;
    blend_att.colorWriteMask =
        VK_COLOR_COMPONENT_R_BIT | VK_COLOR_COMPONENT_G_BIT | VK_COLOR_COMPONENT_B_BIT |
        VK_COLOR_COMPONENT_A_BIT;

    VkPipelineColorBlendStateCreateInfo cb{};
    cb.sType = VK_STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_STATE_CREATE_INFO;
    cb.attachmentCount = 1;
    cb.pAttachments = &blend_att;

    VkDynamicState dyn_states[] = {VK_DYNAMIC_STATE_VIEWPORT, VK_DYNAMIC_STATE_SCISSOR};
    VkPipelineDynamicStateCreateInfo dyn{};
    dyn.sType = VK_STRUCTURE_TYPE_PIPELINE_DYNAMIC_STATE_CREATE_INFO;
    dyn.dynamicStateCount = 2;
    dyn.pDynamicStates = dyn_states;

    VkPushConstantRange push{};
    push.stageFlags = VK_SHADER_STAGE_VERTEX_BIT;
    push.offset = 0;
    push.size = sizeof(float) * 16;

    VkPipelineLayoutCreateInfo plci{};
    plci.sType = VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO;
    plci.setLayoutCount = 1;
    plci.pSetLayouts = &descriptor_set_layout_;
    plci.pushConstantRangeCount = 1;
    plci.pPushConstantRanges = &push;
    if (vkCreatePipelineLayout(device_, &plci, nullptr, &pipeline_layout_) != VK_SUCCESS) {
        vkDestroyShaderModule(device_, vert, nullptr);
        vkDestroyShaderModule(device_, frag, nullptr);
        return false;
    }

    // Render pass format placeholder — recreate pipeline after first swapchain if needed.
    // We create render_pass with B8G8R8A8_UNORM; swapchain usually matches.
    VkGraphicsPipelineCreateInfo pci{};
    pci.sType = VK_STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO;
    pci.stageCount = 2;
    pci.pStages = stages;
    pci.pVertexInputState = &vi;
    pci.pInputAssemblyState = &ia;
    pci.pViewportState = &vp;
    pci.pRasterizationState = &rs;
    pci.pMultisampleState = &ms;
    pci.pColorBlendState = &cb;
    pci.pDynamicState = &dyn;
    pci.layout = pipeline_layout_;
    pci.renderPass = render_pass_;
    pci.subpass = 0;

    const VkResult result = vkCreateGraphicsPipelines(device_, VK_NULL_HANDLE, 1, &pci, nullptr, &pipeline_);
    vkDestroyShaderModule(device_, vert, nullptr);
    vkDestroyShaderModule(device_, frag, nullptr);
    if (result != VK_SUCCESS) {
        vs_log("vkCreateGraphicsPipelines failed");
        return false;
    }
    return true;
}

bool RendererVulkan::create_geometry_buffers() {
    const VkDeviceSize vb_size = sizeof(SpriteVertex) * kMaxSpritesPerFrame * 4;
    if (!create_buffer(
            vb_size,
            VK_BUFFER_USAGE_VERTEX_BUFFER_BIT,
            VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT,
            &vertex_buffer_,
            &vertex_memory_)) {
        return false;
    }
    if (vkMapMemory(device_, vertex_memory_, 0, vb_size, 0, &vertex_mapped_) != VK_SUCCESS) {
        return false;
    }

    std::vector<uint16_t> indices(kMaxSpritesPerBatch * 6);
    for (uint32_t i = 0; i < kMaxSpritesPerBatch; ++i) {
        const uint16_t base = static_cast<uint16_t>(i * 4);
        indices[i * 6 + 0] = base + 0;
        indices[i * 6 + 1] = base + 1;
        indices[i * 6 + 2] = base + 2;
        indices[i * 6 + 3] = base + 2;
        indices[i * 6 + 4] = base + 3;
        indices[i * 6 + 5] = base + 0;
    }
    const VkDeviceSize ib_size = sizeof(uint16_t) * indices.size();
    if (!create_buffer(
            ib_size,
            VK_BUFFER_USAGE_INDEX_BUFFER_BIT,
            VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT,
            &index_buffer_,
            &index_memory_)) {
        return false;
    }
    void* mapped = nullptr;
    vkMapMemory(device_, index_memory_, 0, ib_size, 0, &mapped);
    std::memcpy(mapped, indices.data(), static_cast<size_t>(ib_size));
    vkUnmapMemory(device_, index_memory_);
    return true;
}

void RendererVulkan::destroy_framebuffers() {
    for (VkFramebuffer fb : framebuffers_) {
        if (fb) {
            vkDestroyFramebuffer(device_, fb, nullptr);
        }
    }
    framebuffers_.clear();
}

bool RendererVulkan::create_framebuffers() {
    destroy_framebuffers();
    framebuffers_.resize(swapchain_views_.size());
    for (size_t i = 0; i < swapchain_views_.size(); ++i) {
        VkFramebufferCreateInfo ci{};
        ci.sType = VK_STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO;
        ci.renderPass = render_pass_;
        ci.attachmentCount = 1;
        ci.pAttachments = &swapchain_views_[i];
        ci.width = swapchain_extent_.width;
        ci.height = swapchain_extent_.height;
        ci.layers = 1;
        if (vkCreateFramebuffer(device_, &ci, nullptr, &framebuffers_[i]) != VK_SUCCESS) {
            return false;
        }
    }
    return true;
}

TextureHandle RendererVulkan::create_texture_rgba8(uint32_t width, uint32_t height, const uint8_t* rgba) {
    if (!device_ || !rgba || width == 0 || height == 0) {
        return {};
    }
    const VkDeviceSize size = static_cast<VkDeviceSize>(width) * height * 4;

    VkBuffer staging = VK_NULL_HANDLE;
    VkDeviceMemory staging_mem = VK_NULL_HANDLE;
    if (!create_buffer(
            size,
            VK_BUFFER_USAGE_TRANSFER_SRC_BIT,
            VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT,
            &staging,
            &staging_mem)) {
        return {};
    }
    void* mapped = nullptr;
    vkMapMemory(device_, staging_mem, 0, size, 0, &mapped);
    std::memcpy(mapped, rgba, static_cast<size_t>(size));
    vkUnmapMemory(device_, staging_mem);

    GpuTexture tex{};
    tex.width = width;
    tex.height = height;

    VkImageCreateInfo ici{};
    ici.sType = VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO;
    ici.imageType = VK_IMAGE_TYPE_2D;
    ici.format = VK_FORMAT_R8G8B8A8_UNORM;
    ici.extent = {width, height, 1};
    ici.mipLevels = 1;
    ici.arrayLayers = 1;
    ici.samples = VK_SAMPLE_COUNT_1_BIT;
    ici.tiling = VK_IMAGE_TILING_OPTIMAL;
    ici.usage = VK_IMAGE_USAGE_TRANSFER_DST_BIT | VK_IMAGE_USAGE_SAMPLED_BIT;
    ici.sharingMode = VK_SHARING_MODE_EXCLUSIVE;
    ici.initialLayout = VK_IMAGE_LAYOUT_UNDEFINED;
    if (vkCreateImage(device_, &ici, nullptr, &tex.image) != VK_SUCCESS) {
        vkDestroyBuffer(device_, staging, nullptr);
        vkFreeMemory(device_, staging_mem, nullptr);
        return {};
    }
    VkMemoryRequirements req{};
    vkGetImageMemoryRequirements(device_, tex.image, &req);
    VkMemoryAllocateInfo ai{};
    ai.sType = VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO;
    ai.allocationSize = req.size;
    ai.memoryTypeIndex =
        find_memory_type(req.memoryTypeBits, VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT);
    if (vkAllocateMemory(device_, &ai, nullptr, &tex.memory) != VK_SUCCESS) {
        vkDestroyImage(device_, tex.image, nullptr);
        vkDestroyBuffer(device_, staging, nullptr);
        vkFreeMemory(device_, staging_mem, nullptr);
        return {};
    }
    vkBindImageMemory(device_, tex.image, tex.memory, 0);

    vkWaitForFences(device_, 1, &in_flight_, VK_TRUE, UINT64_MAX);
    vkResetCommandBuffer(command_buffer_, 0);
    VkCommandBufferBeginInfo begin{};
    begin.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO;
    begin.flags = VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT;
    vkBeginCommandBuffer(command_buffer_, &begin);
    transition_image(
        command_buffer_,
        tex.image,
        VK_IMAGE_LAYOUT_UNDEFINED,
        VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
        0,
        VK_ACCESS_TRANSFER_WRITE_BIT,
        VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT,
        VK_PIPELINE_STAGE_TRANSFER_BIT);
    VkBufferImageCopy region{};
    region.imageSubresource.aspectMask = VK_IMAGE_ASPECT_COLOR_BIT;
    region.imageSubresource.layerCount = 1;
    region.imageExtent = {width, height, 1};
    vkCmdCopyBufferToImage(
        command_buffer_,
        staging,
        tex.image,
        VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
        1,
        &region);
    transition_image(
        command_buffer_,
        tex.image,
        VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
        VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL,
        VK_ACCESS_TRANSFER_WRITE_BIT,
        VK_ACCESS_SHADER_READ_BIT,
        VK_PIPELINE_STAGE_TRANSFER_BIT,
        VK_PIPELINE_STAGE_FRAGMENT_SHADER_BIT);
    vkEndCommandBuffer(command_buffer_);
    VkSubmitInfo submit{};
    submit.sType = VK_STRUCTURE_TYPE_SUBMIT_INFO;
    submit.commandBufferCount = 1;
    submit.pCommandBuffers = &command_buffer_;
    vkResetFences(device_, 1, &in_flight_);
    vkQueueSubmit(queue_, 1, &submit, in_flight_);
    vkWaitForFences(device_, 1, &in_flight_, VK_TRUE, UINT64_MAX);

    vkDestroyBuffer(device_, staging, nullptr);
    vkFreeMemory(device_, staging_mem, nullptr);

    VkImageViewCreateInfo vci{};
    vci.sType = VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO;
    vci.image = tex.image;
    vci.viewType = VK_IMAGE_VIEW_TYPE_2D;
    vci.format = VK_FORMAT_R8G8B8A8_UNORM;
    vci.subresourceRange.aspectMask = VK_IMAGE_ASPECT_COLOR_BIT;
    vci.subresourceRange.levelCount = 1;
    vci.subresourceRange.layerCount = 1;
    if (vkCreateImageView(device_, &vci, nullptr, &tex.view) != VK_SUCCESS) {
        destroy_gpu_texture(tex);
        return {};
    }

    VkDescriptorSetAllocateInfo dsai{};
    dsai.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO;
    dsai.descriptorPool = descriptor_pool_;
    dsai.descriptorSetCount = 1;
    dsai.pSetLayouts = &descriptor_set_layout_;
    if (vkAllocateDescriptorSets(device_, &dsai, &tex.descriptor) != VK_SUCCESS) {
        destroy_gpu_texture(tex);
        return {};
    }
    VkDescriptorImageInfo image_info{};
    image_info.sampler = sampler_;
    image_info.imageView = tex.view;
    image_info.imageLayout = VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL;
    VkWriteDescriptorSet write{};
    write.sType = VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET;
    write.dstSet = tex.descriptor;
    write.dstBinding = 0;
    write.descriptorType = VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER;
    write.descriptorCount = 1;
    write.pImageInfo = &image_info;
    vkUpdateDescriptorSets(device_, 1, &write, 0, nullptr);

    textures_.push_back(tex);
    TextureHandle handle{};
    handle.id = static_cast<uint32_t>(textures_.size());
    return handle;
}

void RendererVulkan::destroy_gpu_texture(GpuTexture& tex) {
    if (tex.descriptor && descriptor_pool_) {
        vkFreeDescriptorSets(device_, descriptor_pool_, 1, &tex.descriptor);
        tex.descriptor = VK_NULL_HANDLE;
    }
    if (tex.view) {
        vkDestroyImageView(device_, tex.view, nullptr);
        tex.view = VK_NULL_HANDLE;
    }
    if (tex.image) {
        vkDestroyImage(device_, tex.image, nullptr);
        tex.image = VK_NULL_HANDLE;
    }
    if (tex.memory) {
        vkFreeMemory(device_, tex.memory, nullptr);
        tex.memory = VK_NULL_HANDLE;
    }
}

void RendererVulkan::destroy_texture(TextureHandle handle) {
    if (!handle.valid() || handle.id > textures_.size()) {
        return;
    }
    destroy_gpu_texture(textures_[handle.id - 1]);
}

bool RendererVulkan::begin_frame() {
    if (!device_ || swapchain_ == VK_NULL_HANDLE) {
        return false;
    }
    vkWaitForFences(device_, 1, &in_flight_, VK_TRUE, UINT64_MAX);
    vkResetFences(device_, 1, &in_flight_);

    VkResult result = vkAcquireNextImageKHR(
        device_, swapchain_, UINT64_MAX, image_available_, VK_NULL_HANDLE, &image_index_);
    if (result == VK_ERROR_OUT_OF_DATE_KHR) {
        return false;
    }
    if (result != VK_SUCCESS && result != VK_SUBOPTIMAL_KHR) {
        vs_log("vkAcquireNextImageKHR failed: %d", static_cast<int>(result));
        return false;
    }

    frame_draw_calls_ = 0;
    frame_sprites_ = 0;
    batch_sprite_count_ = 0;
    batch_vertex_base_ = 0;
    batch_texture_ = {};
    frame_open_ = true;

    vkResetCommandBuffer(command_buffer_, 0);
    VkCommandBufferBeginInfo begin{};
    begin.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO;
    begin.flags = VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT;
    vkBeginCommandBuffer(command_buffer_, &begin);

    VkClearValue clear{};
    clear.color.float32[0] = 0.05f;
    clear.color.float32[1] = 0.35f;
    clear.color.float32[2] = 0.40f;
    clear.color.float32[3] = 1.0f;

    VkRenderPassBeginInfo rp{};
    rp.sType = VK_STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO;
    rp.renderPass = render_pass_;
    rp.framebuffer = framebuffers_[image_index_];
    rp.renderArea.extent = swapchain_extent_;
    rp.clearValueCount = 1;
    rp.pClearValues = &clear;
#ifdef VS_GPU_TIMESTAMPS
    if (timestamp_pool_ != VK_NULL_HANDLE) {
        if (timestamps_pending_) {
            uint64_t stamps[2] = {};
            if (vkGetQueryPoolResults(
                    device_,
                    timestamp_pool_,
                    0,
                    2,
                    sizeof(stamps),
                    stamps,
                    sizeof(uint64_t),
                    VK_QUERY_RESULT_64_BIT) == VK_SUCCESS) {
                const double ns = static_cast<double>(stamps[1] - stamps[0]) * timestamp_period_ns_;
                last_gpu_frame_ms_ = ns / 1.0e6;
            }
        }
        vkCmdResetQueryPool(command_buffer_, timestamp_pool_, 0, 2);
        vkCmdWriteTimestamp(command_buffer_, VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT, timestamp_pool_, 0);
    }
#endif

    vkCmdBeginRenderPass(command_buffer_, &rp, VK_SUBPASS_CONTENTS_INLINE);

    VkViewport viewport{};
    viewport.width = static_cast<float>(swapchain_extent_.width);
    viewport.height = static_cast<float>(swapchain_extent_.height);
    viewport.maxDepth = 1.0f;
    vkCmdSetViewport(command_buffer_, 0, 1, &viewport);
    VkRect2D scissor{};
    scissor.extent = swapchain_extent_;
    vkCmdSetScissor(command_buffer_, 0, 1, &scissor);

    vkCmdBindPipeline(command_buffer_, VK_PIPELINE_BIND_POINT_GRAPHICS, pipeline_);
    vkCmdPushConstants(
        command_buffer_,
        pipeline_layout_,
        VK_SHADER_STAGE_VERTEX_BIT,
        0,
        sizeof(view_proj_),
        view_proj_);
    VkDeviceSize offset = 0;
    vkCmdBindVertexBuffers(command_buffer_, 0, 1, &vertex_buffer_, &offset);
    vkCmdBindIndexBuffer(command_buffer_, index_buffer_, 0, VK_INDEX_TYPE_UINT16);
    return true;
}

void RendererVulkan::set_camera(const Camera2D& camera) {
    camera_ = camera;
    mat3_to_mat4(camera_.view_projection(), view_proj_);
    if (frame_open_) {
        vkCmdPushConstants(
            command_buffer_,
            pipeline_layout_,
            VK_SHADER_STAGE_VERTEX_BIT,
            0,
            sizeof(view_proj_),
            view_proj_);
    }
}

void RendererVulkan::flush_batch() {
    if (batch_sprite_count_ == 0 || !batch_texture_.valid()) {
        batch_sprite_count_ = 0;
        return;
    }
    if (batch_texture_.id == 0 || batch_texture_.id > textures_.size()) {
        batch_sprite_count_ = 0;
        return;
    }
    const GpuTexture& tex = textures_[batch_texture_.id - 1];
    vkCmdBindDescriptorSets(
        command_buffer_,
        VK_PIPELINE_BIND_POINT_GRAPHICS,
        pipeline_layout_,
        0,
        1,
        &tex.descriptor,
        0,
        nullptr);
    // Indices are 0..(batch-1) relative; vertexOffset selects this batch's verts.
    vkCmdDrawIndexed(
        command_buffer_,
        batch_sprite_count_ * 6,
        1,
        0,
        static_cast<int32_t>(batch_vertex_base_),
        0);
    ++frame_draw_calls_;
    batch_vertex_base_ += batch_sprite_count_ * 4;
    batch_sprite_count_ = 0;
}

void RendererVulkan::draw_sprites(const SpriteInstance* sprites, uint32_t count) {
    if (!frame_open_ || !sprites || count == 0) {
        return;
    }
    auto* verts = static_cast<SpriteVertex*>(vertex_mapped_);
    for (uint32_t i = 0; i < count; ++i) {
        const SpriteInstance& s = sprites[i];
        if (!s.texture.valid() || s.texture.id > textures_.size()) {
            continue;
        }
        if (batch_texture_.valid() && batch_texture_.id != s.texture.id) {
            flush_batch();
            batch_texture_ = s.texture;
        } else if (!batch_texture_.valid()) {
            batch_texture_ = s.texture;
        }
        if (batch_sprite_count_ >= kMaxSpritesPerBatch ||
            batch_vertex_base_ + (batch_sprite_count_ + 1) * 4 > kMaxSpritesPerFrame * 4) {
            flush_batch();
            batch_texture_ = s.texture;
            if (batch_vertex_base_ + 4 > kMaxSpritesPerFrame * 4) {
                break; // frame budget exhausted
            }
        }

        const uint32_t base = batch_vertex_base_ + batch_sprite_count_ * 4;
        const float x0 = s.x;
        const float y0 = s.y;
        const float x1 = s.x + s.w;
        const float y1 = s.y + s.h;
        verts[base + 0] = {x0, y0, s.u0, s.v0, s.r, s.g, s.b, s.a};
        verts[base + 1] = {x1, y0, s.u1, s.v0, s.r, s.g, s.b, s.a};
        verts[base + 2] = {x1, y1, s.u1, s.v1, s.r, s.g, s.b, s.a};
        verts[base + 3] = {x0, y1, s.u0, s.v1, s.r, s.g, s.b, s.a};
        ++batch_sprite_count_;
        ++frame_sprites_;
    }
    // Flush before caller changes camera / starts another pass.
    flush_batch();
}

bool RendererVulkan::end_frame() {
    if (!frame_open_) {
        return false;
    }
    flush_batch();
    vkCmdEndRenderPass(command_buffer_);
#ifdef VS_GPU_TIMESTAMPS
    if (timestamp_pool_ != VK_NULL_HANDLE) {
        vkCmdWriteTimestamp(command_buffer_, VK_PIPELINE_STAGE_BOTTOM_OF_PIPE_BIT, timestamp_pool_, 1);
        timestamps_pending_ = true;
    }
#endif
    vkEndCommandBuffer(command_buffer_);

    VkPipelineStageFlags wait_stage = VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT;
    VkSemaphore render_finished = render_finished_[image_index_];
    VkSubmitInfo submit{};
    submit.sType = VK_STRUCTURE_TYPE_SUBMIT_INFO;
    submit.waitSemaphoreCount = 1;
    submit.pWaitSemaphores = &image_available_;
    submit.pWaitDstStageMask = &wait_stage;
    submit.commandBufferCount = 1;
    submit.pCommandBuffers = &command_buffer_;
    submit.signalSemaphoreCount = 1;
    submit.pSignalSemaphores = &render_finished;
    if (vkQueueSubmit(queue_, 1, &submit, in_flight_) != VK_SUCCESS) {
        vs_log("vkQueueSubmit failed");
        frame_open_ = false;
        return false;
    }

    VkPresentInfoKHR present{};
    present.sType = VK_STRUCTURE_TYPE_PRESENT_INFO_KHR;
    present.waitSemaphoreCount = 1;
    present.pWaitSemaphores = &render_finished;
    present.swapchainCount = 1;
    present.pSwapchains = &swapchain_;
    present.pImageIndices = &image_index_;
    VkResult result = vkQueuePresentKHR(queue_, &present);
    frame_open_ = false;
    last_draw_calls_ = frame_draw_calls_;
    last_sprites_ = frame_sprites_;
    if (result != VK_SUCCESS && result != VK_SUBOPTIMAL_KHR && result != VK_ERROR_OUT_OF_DATE_KHR) {
        vs_log("vkQueuePresentKHR failed: %d", static_cast<int>(result));
        return false;
    }
    ++frames_presented_;
    return true;
}

bool RendererVulkan::create_instance(bool validation) {
    VkApplicationInfo app{};
    app.sType = VK_STRUCTURE_TYPE_APPLICATION_INFO;
    app.pApplicationName = "VERTICAL SHIFT";
    app.applicationVersion = VK_MAKE_VERSION(0, 2, 0);
    app.pEngineName = "VerticalShiftRuntime";
    app.engineVersion = VK_MAKE_VERSION(0, 2, 0);
    app.apiVersion = VK_API_VERSION_1_1;

    std::vector<const char*> extensions = {
        VK_KHR_SURFACE_EXTENSION_NAME,
#ifdef __ANDROID__
        VK_KHR_ANDROID_SURFACE_EXTENSION_NAME,
#else
        VK_KHR_WIN32_SURFACE_EXTENSION_NAME,
#endif
    };
    std::vector<const char*> layers;
    if (validation) {
        extensions.push_back(VK_EXT_DEBUG_UTILS_EXTENSION_NAME);
        layers.push_back("VK_LAYER_KHRONOS_validation");
    }

    VkInstanceCreateInfo ci{};
    ci.sType = VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO;
    ci.pApplicationInfo = &app;
    ci.enabledExtensionCount = static_cast<uint32_t>(extensions.size());
    ci.ppEnabledExtensionNames = extensions.data();
    ci.enabledLayerCount = static_cast<uint32_t>(layers.size());
    ci.ppEnabledLayerNames = layers.data();

    VkDebugUtilsMessengerCreateInfoEXT debug_ci{};
    if (validation) {
        debug_ci.sType = VK_STRUCTURE_TYPE_DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT;
        debug_ci.messageSeverity =
            VK_DEBUG_UTILS_MESSAGE_SEVERITY_WARNING_BIT_EXT |
            VK_DEBUG_UTILS_MESSAGE_SEVERITY_ERROR_BIT_EXT;
        debug_ci.messageType =
            VK_DEBUG_UTILS_MESSAGE_TYPE_GENERAL_BIT_EXT |
            VK_DEBUG_UTILS_MESSAGE_TYPE_VALIDATION_BIT_EXT |
            VK_DEBUG_UTILS_MESSAGE_TYPE_PERFORMANCE_BIT_EXT;
        debug_ci.pfnUserCallback = debug_callback;
        ci.pNext = &debug_ci;
    }

    if (vkCreateInstance(&ci, nullptr, &instance_) != VK_SUCCESS) {
        if (validation) {
            vs_log("Validation layers unavailable; retrying without them");
            validation_ = false;
            return create_instance(false);
        }
        vs_log("vkCreateInstance failed");
        return false;
    }

    if (validation_) {
        auto create = reinterpret_cast<PFN_vkCreateDebugUtilsMessengerEXT>(
            vkGetInstanceProcAddr(instance_, "vkCreateDebugUtilsMessengerEXT"));
        if (create) {
            create(instance_, &debug_ci, nullptr, &debug_messenger_);
        }
    }
    return true;
}

bool RendererVulkan::create_surface(void* hwnd, void* hinstance) {
#ifdef __ANDROID__
    (void)hinstance;
    VkAndroidSurfaceCreateInfoKHR ci{};
    ci.sType = VK_STRUCTURE_TYPE_ANDROID_SURFACE_CREATE_INFO_KHR;
    ci.window = static_cast<ANativeWindow*>(hwnd);
    if (vkCreateAndroidSurfaceKHR(instance_, &ci, nullptr, &surface_) != VK_SUCCESS) {
        vs_log("vkCreateAndroidSurfaceKHR failed");
        return false;
    }
#else
    VkWin32SurfaceCreateInfoKHR ci{};
    ci.sType = VK_STRUCTURE_TYPE_WIN32_SURFACE_CREATE_INFO_KHR;
    ci.hwnd = static_cast<HWND>(hwnd);
    ci.hinstance = static_cast<HINSTANCE>(hinstance);
    if (vkCreateWin32SurfaceKHR(instance_, &ci, nullptr, &surface_) != VK_SUCCESS) {
        vs_log("vkCreateWin32SurfaceKHR failed");
        return false;
    }
#endif
    return true;
}

void RendererVulkan::lose_window_surface() {
    if (device_) {
        vkDeviceWaitIdle(device_);
    }
    destroy_framebuffers();
    destroy_swapchain_semaphores();
    destroy_swapchain();
    if (surface_ != VK_NULL_HANDLE && instance_ != VK_NULL_HANDLE) {
        vkDestroySurfaceKHR(instance_, surface_, nullptr);
        surface_ = VK_NULL_HANDLE;
    }
}

bool RendererVulkan::recreate_window_surface(void* window_handle, int width, int height) {
    lose_window_surface();
    if (!window_handle || width <= 0 || height <= 0) {
        return false;
    }
    if (!create_surface(window_handle, module_handle_)) {
        return false;
    }
    // Re-query queue family for the new surface.
    queue_family_ = find_queue_family(physical_, surface_);
    if (queue_family_ == UINT32_MAX) {
        vs_log("recreate_window_surface: no present queue");
        return false;
    }
    vkGetDeviceQueue(device_, queue_family_, 0, &queue_);
    if (!create_swapchain(width, height)) {
        return false;
    }
    camera_.set_viewport(static_cast<float>(width), static_cast<float>(height));
    return true;
}

uint32_t RendererVulkan::find_queue_family(VkPhysicalDevice device, VkSurfaceKHR surface) const {
    uint32_t count = 0;
    vkGetPhysicalDeviceQueueFamilyProperties(device, &count, nullptr);
    std::vector<VkQueueFamilyProperties> props(count);
    vkGetPhysicalDeviceQueueFamilyProperties(device, &count, props.data());
    for (uint32_t i = 0; i < count; ++i) {
        VkBool32 present = VK_FALSE;
        vkGetPhysicalDeviceSurfaceSupportKHR(device, i, surface, &present);
        if ((props[i].queueFlags & VK_QUEUE_GRAPHICS_BIT) && present) {
            return i;
        }
    }
    return UINT32_MAX;
}

bool RendererVulkan::pick_device() {
    uint32_t count = 0;
    vkEnumeratePhysicalDevices(instance_, &count, nullptr);
    if (count == 0) {
        vs_log("No Vulkan physical devices");
        return false;
    }
    std::vector<VkPhysicalDevice> devices(count);
    vkEnumeratePhysicalDevices(instance_, &count, devices.data());
    for (VkPhysicalDevice candidate : devices) {
        uint32_t family = find_queue_family(candidate, surface_);
        if (family == UINT32_MAX) {
            continue;
        }
        uint32_t ext_count = 0;
        vkEnumerateDeviceExtensionProperties(candidate, nullptr, &ext_count, nullptr);
        std::vector<VkExtensionProperties> exts(ext_count);
        vkEnumerateDeviceExtensionProperties(candidate, nullptr, &ext_count, exts.data());
        bool has_swapchain = false;
        for (const auto& e : exts) {
            if (std::strcmp(e.extensionName, VK_KHR_SWAPCHAIN_EXTENSION_NAME) == 0) {
                has_swapchain = true;
                break;
            }
        }
        if (!has_swapchain) {
            continue;
        }
        physical_ = candidate;
        queue_family_ = family;
        VkPhysicalDeviceProperties props{};
        vkGetPhysicalDeviceProperties(physical_, &props);
        vs_log("Using GPU: %s", props.deviceName);
        return true;
    }
    vs_log("No suitable Vulkan device");
    return false;
}

bool RendererVulkan::create_device() {
    float priority = 1.0f;
    VkDeviceQueueCreateInfo qci{};
    qci.sType = VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO;
    qci.queueFamilyIndex = queue_family_;
    qci.queueCount = 1;
    qci.pQueuePriorities = &priority;
    const char* extensions[] = {VK_KHR_SWAPCHAIN_EXTENSION_NAME};
    VkDeviceCreateInfo ci{};
    ci.sType = VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO;
    ci.queueCreateInfoCount = 1;
    ci.pQueueCreateInfos = &qci;
    ci.enabledExtensionCount = 1;
    ci.ppEnabledExtensionNames = extensions;
    if (vkCreateDevice(physical_, &ci, nullptr, &device_) != VK_SUCCESS) {
        vs_log("vkCreateDevice failed");
        return false;
    }
    vkGetDeviceQueue(device_, queue_family_, 0, &queue_);
    return true;
}

bool RendererVulkan::create_swapchain(int width, int height) {
    VkSurfaceCapabilitiesKHR caps{};
    vkGetPhysicalDeviceSurfaceCapabilitiesKHR(physical_, surface_, &caps);

    uint32_t format_count = 0;
    vkGetPhysicalDeviceSurfaceFormatsKHR(physical_, surface_, &format_count, nullptr);
    std::vector<VkSurfaceFormatKHR> formats(format_count);
    vkGetPhysicalDeviceSurfaceFormatsKHR(physical_, surface_, &format_count, formats.data());
    VkSurfaceFormatKHR chosen = formats[0];
    for (const auto& f : formats) {
        if (f.format == VK_FORMAT_B8G8R8A8_UNORM &&
            f.colorSpace == VK_COLOR_SPACE_SRGB_NONLINEAR_KHR) {
            chosen = f;
            break;
        }
    }
    swapchain_format_ = chosen.format;

    if (caps.currentExtent.width != UINT32_MAX) {
        swapchain_extent_ = caps.currentExtent;
    } else {
        swapchain_extent_.width = std::clamp(
            static_cast<uint32_t>(width), caps.minImageExtent.width, caps.maxImageExtent.width);
        swapchain_extent_.height = std::clamp(
            static_cast<uint32_t>(height),
            caps.minImageExtent.height,
            caps.maxImageExtent.height);
    }

    uint32_t image_count = caps.minImageCount + 1;
    if (caps.maxImageCount > 0 && image_count > caps.maxImageCount) {
        image_count = caps.maxImageCount;
    }

    VkSwapchainCreateInfoKHR ci{};
    ci.sType = VK_STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR;
    ci.surface = surface_;
    ci.minImageCount = image_count;
    ci.imageFormat = swapchain_format_;
    ci.imageColorSpace = chosen.colorSpace;
    ci.imageExtent = swapchain_extent_;
    ci.imageArrayLayers = 1;
    ci.imageUsage = VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT;
    ci.imageSharingMode = VK_SHARING_MODE_EXCLUSIVE;
    ci.preTransform = caps.currentTransform;
    ci.compositeAlpha = VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR;
    ci.presentMode = VK_PRESENT_MODE_FIFO_KHR;
    ci.clipped = VK_TRUE;
    if (vkCreateSwapchainKHR(device_, &ci, nullptr, &swapchain_) != VK_SUCCESS) {
        vs_log("vkCreateSwapchainKHR failed");
        return false;
    }

    uint32_t actual = 0;
    vkGetSwapchainImagesKHR(device_, swapchain_, &actual, nullptr);
    swapchain_images_.resize(actual);
    vkGetSwapchainImagesKHR(device_, swapchain_, &actual, swapchain_images_.data());
    swapchain_views_.resize(actual);
    for (uint32_t i = 0; i < actual; ++i) {
        VkImageViewCreateInfo vci{};
        vci.sType = VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO;
        vci.image = swapchain_images_[i];
        vci.viewType = VK_IMAGE_VIEW_TYPE_2D;
        vci.format = swapchain_format_;
        vci.subresourceRange.aspectMask = VK_IMAGE_ASPECT_COLOR_BIT;
        vci.subresourceRange.levelCount = 1;
        vci.subresourceRange.layerCount = 1;
        if (vkCreateImageView(device_, &vci, nullptr, &swapchain_views_[i]) != VK_SUCCESS) {
            return false;
        }
    }
    if (!create_swapchain_semaphores() || !create_framebuffers()) {
        return false;
    }
    return true;
}

void RendererVulkan::destroy_swapchain_semaphores() {
    for (VkSemaphore sem : render_finished_) {
        if (sem) {
            vkDestroySemaphore(device_, sem, nullptr);
        }
    }
    render_finished_.clear();
}

bool RendererVulkan::create_swapchain_semaphores() {
    destroy_swapchain_semaphores();
    render_finished_.assign(swapchain_images_.size(), VK_NULL_HANDLE);
    VkSemaphoreCreateInfo sci{VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO};
    for (size_t i = 0; i < render_finished_.size(); ++i) {
        if (vkCreateSemaphore(device_, &sci, nullptr, &render_finished_[i]) != VK_SUCCESS) {
            return false;
        }
    }
    return true;
}

void RendererVulkan::destroy_swapchain() {
    destroy_framebuffers();
    destroy_swapchain_semaphores();
    for (VkImageView view : swapchain_views_) {
        if (view) {
            vkDestroyImageView(device_, view, nullptr);
        }
    }
    swapchain_views_.clear();
    swapchain_images_.clear();
    if (swapchain_) {
        vkDestroySwapchainKHR(device_, swapchain_, nullptr);
        swapchain_ = VK_NULL_HANDLE;
    }
}

bool RendererVulkan::create_command_pool() {
    VkCommandPoolCreateInfo pci{};
    pci.sType = VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO;
    pci.queueFamilyIndex = queue_family_;
    pci.flags = VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT;
    if (vkCreateCommandPool(device_, &pci, nullptr, &command_pool_) != VK_SUCCESS) {
        return false;
    }
    VkCommandBufferAllocateInfo ai{};
    ai.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO;
    ai.commandPool = command_pool_;
    ai.level = VK_COMMAND_BUFFER_LEVEL_PRIMARY;
    ai.commandBufferCount = 1;
    return vkAllocateCommandBuffers(device_, &ai, &command_buffer_) == VK_SUCCESS;
}

bool RendererVulkan::create_frame_sync() {
    VkSemaphoreCreateInfo sci{VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO};
    VkFenceCreateInfo fci{VK_STRUCTURE_TYPE_FENCE_CREATE_INFO};
    fci.flags = VK_FENCE_CREATE_SIGNALED_BIT;
    return vkCreateSemaphore(device_, &sci, nullptr, &image_available_) == VK_SUCCESS &&
           vkCreateFence(device_, &fci, nullptr, &in_flight_) == VK_SUCCESS;
}
