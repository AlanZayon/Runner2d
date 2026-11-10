#pragma once

#include "../renderer.hpp"

#include <volk.h>

#include <string>
#include <vector>

struct SpriteVertex {
    float x, y;
    float u, v;
    float r, g, b, a;
};

struct GpuTexture {
    VkImage image = VK_NULL_HANDLE;
    VkDeviceMemory memory = VK_NULL_HANDLE;
    VkImageView view = VK_NULL_HANDLE;
    VkDescriptorSet descriptor = VK_NULL_HANDLE;
    uint32_t width = 0;
    uint32_t height = 0;
};

class RendererVulkan final : public Renderer {
public:
    bool init(const RendererCreateInfo& info) override;
    void shutdown() override;
    bool resize(int width, int height) override;
    bool recreate_window_surface(void* window_handle, int width, int height) override;
    void lose_window_surface() override;
    bool has_surface() const override { return surface_ != VK_NULL_HANDLE && swapchain_ != VK_NULL_HANDLE; }
    TextureHandle create_texture_rgba8(uint32_t width, uint32_t height, const uint8_t* rgba) override;
    void destroy_texture(TextureHandle handle) override;
    bool begin_frame() override;
    void set_camera(const Camera2D& camera) override;
    void draw_sprites(const SpriteInstance* sprites, uint32_t count) override;
    bool end_frame() override;
    uint64_t frames_presented() const override { return frames_presented_; }
    uint32_t last_frame_draw_calls() const override { return last_draw_calls_; }
    uint32_t last_frame_sprites() const override { return last_sprites_; }
    double last_gpu_frame_ms() const override { return last_gpu_frame_ms_; }

private:
    /** Sprites writable in one flush batch (index buffer covers this many). */
    static constexpr uint32_t kMaxSpritesPerBatch = 2048;
    /** Total sprites per frame across multiple texture/camera flushes. */
    static constexpr uint32_t kMaxSpritesPerFrame = 8192;

    bool create_instance(bool validation);
    bool pick_device();
    bool create_device();
    bool create_surface(void* hwnd, void* hinstance);
    bool create_swapchain(int width, int height);
    bool create_command_pool();
    bool create_frame_sync();
    bool create_render_pass();
    bool create_framebuffers();
    bool create_descriptors();
    bool create_pipeline();
    bool create_geometry_buffers();
    bool create_swapchain_semaphores();
    void destroy_swapchain_semaphores();
    void destroy_framebuffers();
    void destroy_swapchain();
    void destroy_gpu_texture(GpuTexture& tex);
    void flush_batch();
    bool load_shader_module(const char* filename, VkShaderModule* out);
    uint32_t find_memory_type(uint32_t type_bits, VkMemoryPropertyFlags flags) const;
    uint32_t find_queue_family(VkPhysicalDevice device, VkSurfaceKHR surface) const;
    bool create_buffer(
        VkDeviceSize size,
        VkBufferUsageFlags usage,
        VkMemoryPropertyFlags mem_flags,
        VkBuffer* buffer,
        VkDeviceMemory* memory);
    void mat3_to_mat4(const Mat3& m, float out16[16]) const;

    std::string shader_dir_;
    void* module_handle_ = nullptr;

    VkInstance instance_ = VK_NULL_HANDLE;
    VkDebugUtilsMessengerEXT debug_messenger_ = VK_NULL_HANDLE;
    VkSurfaceKHR surface_ = VK_NULL_HANDLE;
    VkPhysicalDevice physical_ = VK_NULL_HANDLE;
    VkDevice device_ = VK_NULL_HANDLE;
    VkQueue queue_ = VK_NULL_HANDLE;
    uint32_t queue_family_ = 0;
    VkSwapchainKHR swapchain_ = VK_NULL_HANDLE;
    VkFormat swapchain_format_ = VK_FORMAT_UNDEFINED;
    VkExtent2D swapchain_extent_{};
    std::vector<VkImage> swapchain_images_;
    std::vector<VkImageView> swapchain_views_;
    std::vector<VkFramebuffer> framebuffers_;
    std::vector<VkSemaphore> render_finished_;

    VkRenderPass render_pass_ = VK_NULL_HANDLE;
    VkDescriptorSetLayout descriptor_set_layout_ = VK_NULL_HANDLE;
    VkDescriptorPool descriptor_pool_ = VK_NULL_HANDLE;
    VkPipelineLayout pipeline_layout_ = VK_NULL_HANDLE;
    VkPipeline pipeline_ = VK_NULL_HANDLE;
    VkSampler sampler_ = VK_NULL_HANDLE;

    VkBuffer vertex_buffer_ = VK_NULL_HANDLE;
    VkDeviceMemory vertex_memory_ = VK_NULL_HANDLE;
    VkBuffer index_buffer_ = VK_NULL_HANDLE;
    VkDeviceMemory index_memory_ = VK_NULL_HANDLE;
    void* vertex_mapped_ = nullptr;

    VkCommandPool command_pool_ = VK_NULL_HANDLE;
    VkCommandBuffer command_buffer_ = VK_NULL_HANDLE;
    VkSemaphore image_available_ = VK_NULL_HANDLE;
    VkFence in_flight_ = VK_NULL_HANDLE;
    uint32_t image_index_ = 0;
    uint64_t frames_presented_ = 0;
    bool validation_ = false;
    bool frame_open_ = false;

    Camera2D camera_{};
    float view_proj_[16]{};

    std::vector<GpuTexture> textures_; // index = handle.id - 1
    TextureHandle batch_texture_{};
    uint32_t batch_sprite_count_ = 0;
    /** Vertex index where the current batch starts (grows each flush). */
    uint32_t batch_vertex_base_ = 0;
    uint32_t frame_draw_calls_ = 0;
    uint32_t frame_sprites_ = 0;
    uint32_t last_draw_calls_ = 0;
    uint32_t last_sprites_ = 0;

#ifdef VS_GPU_TIMESTAMPS
    VkQueryPool timestamp_pool_ = VK_NULL_HANDLE;
    float timestamp_period_ns_ = 1.0f;
    bool timestamps_pending_ = false;
#endif
    double last_gpu_frame_ms_ = -1.0;
};
