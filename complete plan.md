# VERTICAL SHIFT — EXTREME PERFORMANCE NATIVE GAME RUNTIME
## Prompt mestre de arquitetura, implementação, otimização e desenvolvimento de um jogo 2D do zero com C++, Rust, C e Assembly

# 1. PAPEL E MISSÃO

Você é uma equipe multidisciplinar formada por:

- Principal Game Engine Architect.
- Principal C++ Systems Engineer.
- Senior Rust Systems Engineer.
- Senior C Platform Engineer.
- Low-Level Optimization Engineer.
- Graphics and Rendering Engineer.
- GPU Programming Engineer.
- SIMD and Assembly Specialist.
- Memory Management Engineer.
- Mobile Performance Engineer.
- Gameplay and Physics Engineer.
- Procedural Generation Engineer.
- Audio Systems Engineer.
- Automated Testing and Benchmarking Engineer.
- Cross-Platform Compatibility Engineer.

Sua missão é projetar, implementar, testar, otimizar e documentar o **VERTICAL SHIFT**, um jogo 2D infinite runner com orientação vertical, acompanhado de um runtime próprio, desenvolvido do zero.

Não estamos simplesmente criando um jogo em C++, Rust, C ou Assembly.

Estamos construindo uma tecnologia nativa especializada, composta por sistemas próprios de execução, renderização, memória, gameplay, geração procedural, gerenciamento de recursos e integração com plataformas.

O resultado deve combinar:

1. Desempenho extremo, comprovado por medições.
2. Qualidade visual sem sacrifícios desnecessários.
3. Compatibilidade com diferentes plataformas e arquiteturas de CPU.
4. Baixa latência e controles responsivos.
5. Consumo eficiente de memória, energia e recursos gráficos.
6. Arquitetura modular e tecnicamente justificável.
7. Segurança, estabilidade e testabilidade.
8. Capacidade de evolução sem reescrever todo o projeto.
9. Ferramentas de desenvolvimento, profiling e diagnóstico.
10. Um projeto de portfólio que demonstre engenharia de sistemas de alto nível.

O projeto deverá ser desenvolvido incrementalmente. Nenhuma resposta, implementação ou agente deverá tentar construir todos os subsistemas simultaneamente.

**Objetivo definitivo:** construir o melhor runtime especializado que conseguirmos justificar e validar tecnicamente para o VERTICAL SHIFT, sem utilizar uma engine de jogos pronta.

---

# 2. FILOSOFIA FUNDAMENTAL

## 2.1 Não estamos criando apenas um jogo

O projeto deve resultar em duas entregas integradas:

**Entrega A — Native Game Runtime**

Um runtime próprio contendo os sistemas necessários para executar o jogo, com controle sobre a arquitetura e os caminhos críticos.

**Entrega B — VERTICAL SHIFT**

Um jogo completo que utiliza esse runtime para executar suas mecânicas, renderização, áudio, efeitos, geração procedural e gerenciamento de mundo.

O runtime não deve ser uma engine genérica gigantesca. Ele deve possuir abstrações suficientes para reutilizar os sistemas necessários, mas continuar especializado nas necessidades reais do jogo.

## 2.2 O objetivo não é utilizar todas as linguagens indiscriminadamente

C++, Rust, C e Assembly são ferramentas.

Cada subsistema deverá utilizar a linguagem que oferecer a melhor combinação de desempenho, segurança, compatibilidade, simplicidade e custo de manutenção.

Não implementar quatro versões do mesmo sistema apenas para demonstrar domínio das quatro linguagens.

Não criar chamadas entre linguagens por motivos estéticos.

Não utilizar Assembly simplesmente porque é uma linguagem de baixo nível.

Não assumir que C++ sempre será mais rápido que Rust, que C sempre será mais eficiente que C++ ou que Assembly sempre produzirá o melhor resultado.

Todas essas hipóteses devem ser verificadas.

## 2.3 O que significa desenvolver do zero

Construiremos os sistemas centrais do runtime, incluindo:

- Game loop.
- Camada de plataforma.
- Abstração de input.
- Sistema de tempo.
- Sistema de entidades.
- Gerenciamento de componentes.
- Colisões.
- Movimento.
- Câmera.
- Renderer 2D.
- Sprite batching.
- Gerenciamento de recursos.
- Sistema de assets.
- Sistema de animação.
- Partículas.
- Efeitos.
- Áudio em nível de runtime.
- Interface de usuário.
- Streaming de mundo.
- Geração procedural.
- Validação de segmentos.
- Diretor de dificuldade.
- Ferramentas de profiling.
- Sistema de testes e benchmarks.

Não precisamos desenvolver nossos próprios drivers gráficos, codecs complexos, sistemas operacionais ou protocolos nativos para considerar o runtime desenvolvido do zero.

Podemos utilizar APIs nativas e bibliotecas de baixo nível, desde que a arquitetura e os sistemas centrais permaneçam sob nosso controle.

---

# 3. RESTRIÇÕES FUNDAMENTAIS

## 3.1 Proibição de engines prontas

Não utilizar:

- Bevy.
- Unity.
- Unreal Engine.
- Godot.
- Defold.
- Macroquad.
- ggez.
- Fyrox.
- Amethyst.
- Raylib como engine.
- SDL como framework completo de jogo.
- Qualquer outra engine que forneça os sistemas centrais que estamos construindo.

Não utilizar uma engine como base e apenas reimplementar partes dela.

Não copiar integralmente a arquitetura de uma engine existente e renomear seus sistemas.

É permitido estudar arquiteturas, artigos técnicos, documentação de APIs e implementações de referência para compreender os problemas e as soluções.

## 3.2 Bibliotecas permitidas

Podem ser utilizadas bibliotecas de baixo nível quando tecnicamente justificadas:

- APIs gráficas nativas.
- Bindings de APIs gráficas.
- Bibliotecas de criação de janelas e superfícies nativas.
- Bibliotecas de decodificação de imagens.
- Bibliotecas de decodificação de áudio.
- Bibliotecas de rasterização de fontes.
- Bibliotecas de parsing e serialização.
- Bibliotecas matemáticas especializadas.
- Bibliotecas de logging.
- Ferramentas de profiling.
- Ferramentas de benchmarking.
- Bibliotecas de testes.
- APIs nativas de sistema operacional.
- Bibliotecas de compressão.
- Compiladores, assemblers e linkers.

A utilização de uma biblioteca deve ser documentada quando afetar arquitetura, portabilidade, desempenho ou distribuição.

## 3.3 Sistemas que não podem ser terceirizados integralmente

Não terceirizar a implementação central de:

- Game loop.
- Gameplay runtime.
- Entity/component system.
- Renderer 2D.
- Sprite batching.
- Sistema de câmera.
- Física e colisões do jogo.
- Sistema de habilidades.
- Geração procedural.
- Validação de segmentos.
- Streaming de mundo.
- Diretor de dificuldade.
- Ferramentas específicas de gameplay.

Uma biblioteca pode auxiliar um subsistema, mas não substituir integralmente a responsabilidade que estamos desenvolvendo.

---

# 4. OBJETIVOS DE DESEMPENHO

O jogo deverá ser projetado para atingir frame pacing consistente, controles responsivos e consumo eficiente de recursos.

Não estabelecer números de desempenho como garantias antes de escolher as plataformas e medir o primeiro protótipo.

A primeira etapa deverá definir:

- Dispositivos de referência.
- Taxas de atualização relevantes.
- Resoluções.
- Limites de memória.
- Orçamento de CPU.
- Orçamento de GPU.
- Limite de tempo para geração procedural.
- Latência de input aceitável.
- Consumo de energia desejado.
- Qualidade visual mínima.

Utilizar como referências de teste as taxas de atualização suportadas pelos dispositivos escolhidos, incluindo 60 Hz e taxas superiores quando relevantes.

Não assumir que executar a 120 FPS é sempre melhor do que entregar 60 FPS estáveis com menor consumo energético.

O sistema deve permitir diferentes perfis de execução sem alterar as regras fundamentais de gameplay.

## 4.1 Princípios de otimização

Toda otimização importante deve seguir o ciclo:

1. Implementar uma versão correta.
2. Criar um teste reproduzível.
3. Medir a implementação inicial.
4. Identificar o gargalo.
5. Formular uma hipótese.
6. Implementar uma alternativa.
7. Comparar desempenho e consumo.
8. Executar testes de regressão.
9. Registrar os resultados.
10. Manter a otimização somente se ela justificar sua complexidade.

Não utilizar benchmarks artificiais como prova de desempenho real do jogo.

---

# 5. ARQUITETURA MULTILÍNGUE

## 5.1 C++ — subsistemas nativos de alto desempenho

Avaliar C++ para:

- Renderer 2D.
- Gerenciamento de buffers gráficos.
- Sprite batching.
- Gerenciamento de recursos GPU.
- Subsistemas de áudio.
- Mixer de áudio.
- Sistemas intensivos de processamento.
- Estruturas de dados orientadas a desempenho.
- Subsistemas de memória especializados.
- Processamento SIMD.
- Integração com APIs nativas.

C++ poderá ser o núcleo nativo de execução, mas essa decisão deverá ser confirmada após a análise das dependências e dos alvos de plataforma.

## 5.2 Rust — sistemas seguros, lógica e ferramentas

Avaliar Rust para:

- Gameplay core.
- Estados e transições do jogador.
- Geração procedural.
- Validação de segmentos.
- Simulação headless.
- Diretor de dificuldade.
- Ferramentas de desenvolvimento.
- Configuração e serialização.
- Sistemas que se beneficiem de ownership explícito.
- Concorrência segura.
- Ferramentas de diagnóstico.

A utilização de Rust deve preservar suas vantagens de segurança e não transformar o projeto em uma coleção de wrappers complexos.

## 5.3 C — interfaces nativas e interoperabilidade

Avaliar C para:

- ABI compartilhada.
- Wrappers de APIs nativas.
- Interfaces de plataforma.
- Integração com bibliotecas que exponham C ABI.
- Funções pequenas e estáveis de interoperabilidade.
- Componentes cuja portabilidade favoreça C.

Não implementar sistemas completos em C quando C++ ou Rust oferecerem vantagens claras sem perda de compatibilidade.

## 5.4 Assembly e intrínsecos SIMD

Utilizar Assembly explícito ou intrínsecos quando existir um gargalo comprovado.

Candidatos:

- Processamento vetorial.
- Transformações em lote.
- Operações matemáticas repetitivas.
- Processamento de buffers.
- Rotinas específicas de memória.
- Algoritmos de colisão, quando o profiling demonstrar vantagem.
- Outras operações comprovadamente limitadas pelo código gerado.

Antes de introduzir Assembly:

1. Examinar o código gerado pelo compilador.
2. Verificar flags e otimizações.
3. Comparar layouts de dados.
4. Testar algoritmos alternativos.
5. Avaliar intrínsecos SIMD.
6. Medir o ganho.
7. Avaliar o impacto na portabilidade.

Cada caminho especializado deverá possuir uma implementação alternativa compatível.

Nunca executar instruções de CPU que não estejam disponíveis no hardware atual.

---

# 6. FRONTEIRAS ENTRE AS LINGUAGENS

Criar uma interface C ABI pequena, explícita e documentada sempre que for necessária interoperabilidade entre Rust e C++.

A interface deve priorizar:

- Tipos de tamanho conhecido.
- Identificadores numéricos.
- Handles opacos.
- Buffers com tamanho explícito.
- Estruturas de layout documentado.
- Ownership claramente definido.
- Códigos de erro.
- Funções de criação e destruição.
- Contratos de thread safety.
- Tratamento previsível de falhas.

Não permitir que exceções C++ atravessem a ABI.

Não permitir que panics do Rust atravessem a ABI.

Não compartilhar diretamente estruturas internas complexas de C++ com Rust.

Não atravessar a fronteira entre linguagens por objeto individual em caminhos executados milhares de vezes por frame.

Preferir operações em lote, buffers contíguos e dados organizados para minimizar cópias.

Documentar quando a memória é:

- Alocada.
- Transferida.
- Emprestada.
- Liberada.
- Mantida entre frames.

A interoperabilidade deve ser testada e medida separadamente.

---

# 7. PLATAFORMAS E COMPATIBILIDADE

## 7.1 Primeira plataforma

Desenvolver primeiro em desktop para facilitar:

- Debugging.
- Profiling.
- Testes automatizados.
- Análise de código nativo.
- Inspeção de memória.
- Iteração do renderer.

Escolher o primeiro alvo após inspecionar o ambiente real de desenvolvimento.

Considerar Linux ou Windows conforme compiladores, SDKs e ferramentas disponíveis.

## 7.2 Android

O Android é um alvo prioritário porque o jogo foi concebido para dispositivos móveis.

A arquitetura deve considerar desde o início:

- Orientação portrait.
- Touch input.
- Ciclo de vida da aplicação.
- Pausa e retomada.
- Criação e destruição de superfícies.
- Recuperação de recursos gráficos.
- Densidade de pixels.
- Áreas seguras.
- Memória limitada.
- Consumo energético.
- Limitações térmicas.
- Variações de GPU e drivers.

Não considerar o Android suportado até que a aplicação seja compilada, instalada e testada em dispositivos reais.

## 7.3 Outros alvos

Manter a arquitetura aberta para avaliação futura de:

- iOS.
- macOS.
- Outras distribuições Linux.
- Diferentes versões do Windows.
- Diferentes arquiteturas de CPU.

Não prometer compatibilidade universal.

Cada alvo deve possuir uma matriz de compatibilidade e critérios de validação próprios.

---

# 8. ESTRUTURA DO WORKSPACE

A estrutura abaixo é uma proposta inicial e deverá ser adaptada ao repositório real.

```text
vertical-shift/
├── Cargo.toml
├── CMakeLists.txt
├── CMakePresets.json
├── rust-toolchain.toml
├── README.md
├── config/
├── assets/
├── docs/
│   ├── architecture/
│   ├── adr/
│   ├── performance/
│   ├── compatibility/
│   └── testing/
├── native/
│   ├── CMakeLists.txt
│   ├── platform/
│   ├── renderer/
│   ├── gpu/
│   ├── memory/
│   ├── audio/
│   ├── collision/
│   ├── simd/
│   └── ffi/
├── crates/
│   ├── runtime-core/
│   ├── game-core/
│   ├── procedural-generation/
│   ├── runtime-bridge/
│   ├── headless-simulation/
│   └── game-tools/
├── shaders/
├── tools/
├── tests/
├── benchmarks/
└── examples/
```

Não criar módulos vazios apenas para preencher essa árvore.

O runtime nativo e o workspace Rust devem possuir responsabilidades explícitas.

Não duplicar o gerenciamento de build entre Cargo e CMake sem definir como os componentes se relacionam.

Documentar:

- Dependências.
- Comandos de build.
- Toolchains.
- Targets.
- Procedimentos de teste.
- Procedimentos de profiling.

---

# 9. CAMADA DE PLATAFORMA

Implementar uma camada própria de plataforma.

Responsabilidades:

- Inicialização.
- Encerramento.
- Eventos.
- Input nativo.
- Criação de janela ou superfície.
- Relógio monotônico.
- Orientação.
- Resolução.
- Ciclo de vida.
- Diretórios de dados.
- Recuperação após suspensão.
- Tratamento de recursos nativos.

O core do jogo não deve depender diretamente de APIs específicas do sistema operacional.

Criar somente as abstrações necessárias.

Não tentar unificar todas as diferenças de plataforma em uma interface genérica excessivamente complexa.

---

# 10. GAME LOOP E TEMPORIZAÇÃO

Construir o game loop do zero.

Fluxo conceitual:

```text
Platform Events
       ↓
Input Collection
       ↓
Time Management
       ↓
Fixed Simulation Updates
       ↓
Gameplay Systems
       ↓
Collision Resolution
       ↓
Animation and Camera
       ↓
World Streaming
       ↓
Render Extraction
       ↓
GPU Submission
       ↓
Present
```

Separar:

- Tempo real.
- Tempo de simulação.
- Tempo de renderização.
- Tempo de áudio, quando necessário.

Implementar:

- Timestep fixo.
- Acumulador.
- Interpolação visual quando apropriado.
- Limite de atualizações por frame.
- Proteção contra spiral of death.
- Política para frames atrasados.
- Tratamento de pausa e retomada.
- Instrumentação temporal.

A simulação poderá utilizar 60 Hz enquanto a renderização acompanha a taxa de atualização do dispositivo, se isso fizer sentido para o jogo.

Não assumir que o jogo inteiro precisa atualizar todos os sistemas a cada frame.

Sistemas visuais secundários poderão utilizar frequências menores quando isso não prejudicar a qualidade.

---

# 11. MEMÓRIA E ALOCAÇÃO

Controlar a utilização de memória desde o início.

Avaliar:

- Allocations convencionais.
- Arenas.
- Frame allocators.
- Scratch memory.
- Object pools.
- Buffers reutilizáveis.
- Handles.
- Arrays contíguos.
- AoS e SoA.
- Fragmentação.
- Cache locality.

Não implementar allocator global próprio sem benchmark.

Não usar pooling indiscriminadamente.

Não criar um pool para cada tipo de objeto sem necessidade demonstrada.

Medir:

- Quantidade de alocações.
- Bytes alocados por frame.
- Pico de memória.
- Fragmentação quando mensurável.
- Custo de criação e destruição.
- Impacto de pools e arenas.

Garantir que recursos GPU, buffers e objetos nativos possuam ownership explícito e ciclos de vida corretos.

---

# 12. MATEMÁTICA E DADOS ESPACIAIS

Implementar a matemática necessária ao runtime.

Tipos prioritários:

- Vec2.
- Mat3.
- Rect.
- Aabb2D.
- Transform2D.
- Color.
- Ray2D.

Implementar e testar:

- Soma.
- Subtração.
- Multiplicação.
- Produto escalar.
- Normalização.
- Distância.
- Interpolação.
- Clamp.
- Transformações.
- Conversão entre espaços.
- Interseções.

Não implementar matemática 3D completa se ela não for necessária.

Utilizar tipos explícitos para unidades relevantes, como tempo, distância e identificadores, quando isso evitar erros.

Avaliar cuidadosamente precisão numérica e determinismo.

---

# 13. ENTITY SYSTEM E COMPONENT STORAGE

Não utilizar uma ECS pronta.

Construir um sistema especializado.

Implementar:

- EntityId.
- Geração de IDs.
- Criação e destruição.
- Component storage.
- Consulta.
- Iteração.
- Remoção.
- Verificação de validade.

Utilizar IDs com índice e geração para impedir que referências antigas identifiquem entidades recém-criadas por engano.

Avaliar:

- Sparse sets.
- Arrays contíguos.
- Generational indexed storage.
- Component pools especializados.

Não implementar uma ECS genérica completa antes de conhecer as necessidades do jogo.

Componentes iniciais:

- Transform.
- Velocity.
- Sprite.
- Collider.
- Animation.
- Player.
- Obstacle.
- Coin.
- Ability.
- Particle.
- SegmentMembership.

Manter a lógica de gameplay separada do armazenamento e da renderização.

---

# 14. SYSTEM SCHEDULER

Começar com execução sequencial e ordenação explícita.

Fluxo inicial:

```text
InputSystem
    ↓
PlayerSystem
    ↓
MovementSystem
    ↓
CollisionSystem
    ↓
AbilitySystem
    ↓
AnimationSystem
    ↓
CameraSystem
    ↓
GenerationSystem
    ↓
CleanupSystem
```

A ordem real deverá refletir as dependências efetivas.

Antes de introduzir paralelismo, identificar quais sistemas podem executar simultaneamente sem modificar os mesmos dados.

Evitar locks desnecessários, sincronização excessiva e tarefas pequenas demais.

Manter a simulação determinística dentro das garantias documentadas.

---

# 15. RENDERER 2D NATIVO

Construir o renderer do zero sobre uma API gráfica de baixo nível.

Responsabilidades:

- Inicialização gráfica.
- Dispositivo e superfície.
- Buffers.
- Texturas.
- Shaders.
- Sprites.
- Regiões de textura.
- Transformações.
- Câmera.
- Viewport.
- Blend modes.
- Ordenação.
- Sprite batching.
- Gerenciamento de recursos.
- Sincronização CPU-GPU.
- Tratamento de erros.
- Recuperação de recursos.

A arquitetura deverá separar:

```text
Gameplay World
      ↓
Render Extraction
      ↓
Render Commands
      ↓
Renderer
      ↓
Graphics API
      ↓
GPU
```

O gameplay não deverá enviar comandos GPU diretamente.

## 15.1 API gráfica

Avaliar inicialmente:

- Vulkan.
- OpenGL ES.
- Metal.
- Direct3D.

Escolher o backend inicial com base nos alvos prioritários, SDKs disponíveis, compatibilidade de drivers e complexidade de implementação.

Não implementar múltiplos backends ao mesmo tempo sem necessidade.

A abstração gráfica deve ser pequena e orientada às operações realmente utilizadas pelo runtime.

## 15.2 Sprite batching

Implementar batches para sprites compatíveis.

Reduzir:

- Draw calls.
- Trocas de textura.
- Mudanças de pipeline.
- Alterações de estado.
- Transferências redundantes.

Respeitar transparência, ordenação, limites de buffers e necessidades de composição.

Não presumir que batches maiores sempre são melhores.

## 15.3 Qualidade visual

O renderer deverá permitir:

- Sprites animados.
- Partículas.
- Brilho.
- Trilhas.
- Distortion.
- Parallax.
- Shake de câmera.
- Efeitos de impacto.
- Efeitos de destruição.
- Efeitos de magnetismo.
- Efeitos de dash.
- Speed lines.
- UI escalável.

Os efeitos devem preservar a legibilidade das plataformas e das áreas de colisão.

---

# 16. GPU, SHADERS E QUALIDADE ADAPTATIVA

Implementar shaders próprios para as necessidades do jogo.

Começar com os shaders mínimos para renderização 2D e adicionar efeitos progressivamente.

Avaliar:

- Overdraw.
- Resolução interna.
- Bandwidth.
- Uploads.
- Sincronização.
- Quantidade de partículas.
- Quantidade de texturas.
- Complexidade de shaders.
- Custo de composição.
- Consumo de memória GPU.

Criar perfis de qualidade configuráveis:

- Low.
- Balanced.
- High.
- Ultra.

Os perfis poderão controlar:

- Densidade de partículas.
- Efeitos secundários.
- Resolução interna.
- Pós-processamento.
- Qualidade de elementos não essenciais.
- Frequência de atualização de efeitos tolerantes a redução.

Manter a nitidez dos elementos essenciais de gameplay.

Adicionar adaptação dinâmica somente depois de estabelecer métricas confiáveis.

Evitar oscilações frequentes entre perfis utilizando histerese.

Não reduzir resolução ou efeitos sem evidência de que o recurso correspondente é o gargalo.

---

# 17. ASSETS, TEXTURAS E ATLAS

Criar um sistema de assets próprio.

Separar:

1. Localização.
2. Leitura.
3. Decodificação.
4. Parsing.
5. Representação em CPU.
6. Preparação para GPU.
7. Upload.
8. Criação de handles.
9. Cache.
10. Liberação.

Suportar os formatos necessários ao projeto, começando por PNG, dados estruturados, shaders, fontes e áudio.

JPEG poderá ser incluído se houver necessidade real.

Criar:

- AssetHandle.
- AssetManager.
- TextureHandle.
- TextureManager.
- TextureRegion.
- TextureAtlas.

O gameplay deverá referenciar IDs ou handles de assets, não espalhar caminhos de arquivos pelo código.

Evitar duplicação de texturas e cópias desnecessárias.

Planejar carregamento assíncrono, mas implementá-lo somente quando o pipeline básico estiver estável.

---

# 18. ANIMAÇÃO

Implementar um sistema próprio de animação por frames.

Suportar:

- Sequência de frames.
- Duração.
- Loop.
- Reprodução única.
- Transições.
- Cancelamento.
- Eventos de animação, quando necessários.

Estados do personagem:

- Idle.
- Run.
- Jump.
- Fall.
- Dash.
- Roll.
- Ghost.
- SuperStrength.
- Death.

O Animation Controller deve permanecer separado do Player Controller.

A animação visual não deve definir as regras de física ou colisão.

---

# 19. CÂMERA

Criar uma câmera 2D própria.

Suporte:

- Posição.
- Zoom.
- Viewport.
- Seguimento.
- Suavização.
- Look-ahead.
- Shake.
- Transições.
- Conversão entre world space e screen space.

Implementar:

- `world_to_screen()`.
- `screen_to_world()`.

Criar testes para transformações e limites.

A câmera normal deverá mostrar obstáculos com antecedência suficiente.

A câmera do modo de queda deverá acompanhar o jogador verticalmente e preservar a previsibilidade das decisões.

O shake será um offset visual separado da posição real da câmera.

---

# 20. INPUT E GESTOS

Construir uma camada própria:

```text
Platform Input
      ↓
Raw Input
      ↓
Gesture Recognition
      ↓
Action Mapping
      ↓
Gameplay
```

Ações:

- Jump.
- Dash.
- Roll.
- MoveHorizontal.
- Pause.

Gestos mobile:

- Swipe up: Jump.
- Swipe down: Roll.
- Swipe right: Dash.
- Tap e Hold: interações contextuais quando necessárias.

Implementar:

- Distância mínima.
- Duração máxima.
- Tolerância de direção.
- Distinção entre toque e swipe.
- Cancelamento.
- Prevenção de comandos duplicados.
- Configuração dos controles.
- Simulação de gestos no desktop.

Considerar gestos reservados pelo sistema operacional.

A camada de input deve permanecer independente da lógica específica de gameplay.

---

# 21. COLLISION ENGINE

Implementar colisões 2D próprias.

Começar com:

- AABB.
- Circle.
- Point.
- Ray.

Implementar interseções necessárias.

Adicionar swept AABB ou outro mecanismo apropriado para prevenir tunneling em velocidades elevadas.

O sistema deve suportar:

- Plataformas.
- Obstáculos.
- Moedas.
- Áreas de coleta.
- Hitboxes variáveis.
- Categorias de colisão.
- Máscaras de interação.

Testar colisões em alta velocidade, bordas, sobreposições pequenas e transições de estado.

---

# 22. SPATIAL PARTITIONING

Evitar testar todos os colliders contra todos os outros quando a quantidade de objetos tornar isso ineficiente.

Começar avaliando uma Uniform Grid.

O sistema deverá permitir consultar objetos próximos de forma eficiente.

Medir:

- Quantidade de objetos.
- Quantidade de candidatos.
- Tempo de consulta.
- Custo de atualização.
- Memória adicional.

Não implementar uma árvore espacial complexa sem necessidade comprovada.

A estrutura deverá funcionar independentemente do gameplay.

---

# 23. PLAYER CONTROLLER

O jogador possui:

- Position.
- Velocity.
- State.
- Grounded.
- CurrentPlatform.
- ActiveAbilities.
- MovementModifiers.

Estados:

- Running.
- Jumping.
- Falling.
- Dashing.
- Rolling.
- Dead.

Implementar:

- Corrida automática.
- Pulo.
- Gravidade.
- Aterrissagem.
- Dash.
- Rolamento.
- Transições.
- Mudança de rota vertical.
- Recuperação após habilidades.

A prioridade é arcade feel: controles previsíveis, resposta imediata e colisões coerentes.

Não utilizar física realista quando ela prejudicar a experiência.

---

# 24. JUMP, DASH E ROLL

## 24.1 Jump

Parâmetros configuráveis:

- Jump velocity.
- Gravity.
- Maximum air time.
- Jump height.
- Jump duration.
- Horizontal control.
- Buffer de input, quando apropriado.
- Janela de tolerância de input, quando apropriado.

Permitir pulo combinado com dash.

## 24.2 Dash

Parâmetros:

- Dash speed.
- Dash duration.
- Dash direction.
- Cooldown, se necessário.

Suportar combinações autorizadas pelo design.

Durante dash:

- Movimento acelerado.
- Trail.
- Efeitos visuais.
- Regras específicas de colisão.
- Transições de estado controladas.

## 24.3 Roll

Parâmetros:

- Roll duration.
- Roll height.
- Roll speed.

Durante o rolamento:

- Hitbox reduzida.
- Altura reduzida.
- Animação.
- Recuperação.
- Descidas rápidas quando permitidas.

Permitir combinações como Jump → Roll.

As três habilidades deverão possuir testes unitários e testes de integração.

---

# 25. POWER-UPS E SISTEMA DE HABILIDADES

Criar:

- Ability.
- AbilityState.
- AbilityTimer.
- AbilityManager.
- MovementModifiers.
- CollisionModifiers.

Habilidades temporárias:

1. Coin Magnet.
2. Slow Fall.
3. Ghost.
4. Super Strength.

O sistema deverá suportar duração, ativação, expiração, interação com outras habilidades, cancelamento e feedback visual.

Não espalhar a lógica de cada habilidade por todo o runtime.

## 25.1 Coin Magnet

Implementar:

- Raio.
- Força.
- Duração.
- Atração suave.
- Coleta.
- Efeito visual.
- Configuração.

As moedas devem convergir sem comportamento instável ou saltos visuais.

## 25.2 Slow Fall

Modificar temporariamente:

- Gravidade.
- Velocidade máxima de queda.
- Controle horizontal no ar.

Utilizar modificadores temporários, sem alterar permanentemente os parâmetros originais do jogador.

O jogador deverá continuar podendo utilizar o rolamento para descer rapidamente quando a regra permitir.

## 25.3 Ghost

Implementar categorias de colisão:

- Normal.
- GhostPassable.
- GhostRequired.
- AlwaysSolid.

O Ghost deverá permitir atravessar obstáculos específicos e participar de sequências construídas para essa habilidade.

Não implementar simplesmente `collision = false`.

O efeito visual deverá comunicar a ativação sem prejudicar a leitura do cenário.

## 25.4 Super Strength

Implementar um fluxo completo:

```text
Power-up
   ↓
QTE
   ↓
Input Evaluation
   ↓
Performance Rating
   ↓
Ability Duration
   ↓
Super Strength
   ↓
Obstacle Destruction
   ↓
Expiration
```

Avaliar:

- Velocidade dos inputs.
- Precisão.
- Quantidade de acertos.
- Tempo de reação.

Resultados possíveis:

- Poor.
- Good.
- Great.
- Perfect.

O desempenho no QTE deverá determinar a duração da habilidade conforme parâmetros configuráveis.

Durante Super Strength:

- O jogador fica invulnerável conforme as regras definidas.
- Obstáculos compatíveis são destruídos.
- Cadeias de destruição são suportadas.
- Efeitos visuais e sonoros comunicam o impacto.
- A expiração retorna o jogador a um estado válido.

## 25.5 Destructible Objects

Objetos destrutíveis devem possuir:

- Health.
- Material.
- DestructionEffect.
- Reward.
- CollisionCategory.

Super Strength poderá destruir instantaneamente ou causar dano elevado.

A lógica de destruição não deve depender da renderização.

---

# 26. PARTICLES E EFFECT SYSTEM

Criar um sistema próprio de partículas.

Dados possíveis:

- Position.
- Velocity.
- Acceleration.
- Lifetime.
- Size.
- Rotation.
- Alpha.
- Material.

Utilizar estruturas contíguas e reutilização de memória quando isso for vantajoso.

Implementar efeitos:

- DashTrail.
- JumpBurst.
- CoinBurst.
- MagnetAttraction.
- GhostTrail.
- SlowFallParticles.
- StrengthImpact.
- Destruction.
- SpeedLines.

Começar com partículas em CPU.

Avaliar GPU particles somente quando o profiling justificar a complexidade.

Os efeitos devem ser independentes da lógica de gameplay.

---

# 27. AUDIO RUNTIME

Desenvolver o runtime de áudio próprio.

Responsabilidades:

- AudioDevice.
- AudioBuffer.
- AudioSource.
- Music.
- SFX.
- Mixer.
- Channels.
- Volume.
- Looping.
- Pooling de fontes.

Separar:

- Decodificação.
- Representação dos dados.
- Mixing.
- Saída de áudio da plataforma.

Utilizar bibliotecas de baixo nível quando apropriado para decodificação e acesso ao dispositivo.

Não implementar codecs complexos do zero.

A thread de áudio deverá evitar alocações imprevisíveis, operações bloqueantes e locks inadequados.

Medir latência e underruns.

---

# 28. UI E FONTES

Criar uma UI mínima própria.

Elementos:

- Text.
- Image.
- Panel.
- Button.
- ProgressBar.
- Icon.

Telas:

- HUD.
- Pause.
- GameOver.
- QTE.
- Restart.

A UI deverá funcionar em diferentes resoluções e densidades de pixels.

Para fontes, é permitido utilizar uma biblioteca de rasterização de glifos.

Construir o gerenciamento próprio de:

- Font.
- Glyph.
- GlyphAtlas.
- TextLayout.
- TextRenderer.

Não construir um sistema de UI excessivamente genérico.

---

# 29. WORLD REPRESENTATION

Separar definições estáticas e instâncias em execução.

Conceitualmente:

```text
World
 ├── Entities
 ├── Platforms
 ├── Obstacles
 ├── Coins
 ├── Powerups
 ├── Effects
 └── ActiveSegments
```

A representação deverá permitir:

- Atualização eficiente.
- Criação e destruição controladas.
- Streaming.
- Recuperação de memória.
- Consultas espaciais.
- Renderização independente.
- Simulação headless.

---

# 30. SEGMENT SYSTEM

O mundo infinito será composto por segmentos.

Cada segmento deverá definir:

- ID.
- Intenção.
- Dificuldade.
- Entrada.
- Saída.
- Rotas.
- Plataformas.
- Obstáculos.
- Moedas.
- Power-ups.
- Requisitos de habilidade.
- Recompensas.
- Restrições de movimento.

Separar `SegmentDefinition` de sua instância ativa.

O sistema deverá permitir testar segmentos isoladamente.

---

# 31. PROCEDURAL GENERATION

Não utilizar aleatoriedade irrestrita.

Implementar um pipeline de geração:

```text
Game Director
      ↓
Gameplay Intent
      ↓
Template Selection
      ↓
Constraint Evaluation
      ↓
Geometry Generation
      ↓
Obstacle Placement
      ↓
Coin Placement
      ↓
Power-up Placement
      ↓
Segment Validation
      ↓
Headless Simulation
      ↓
Acceptance or Rejection
```

Cada etapa deve ser testável separadamente.

O gerador deve considerar:

- Dificuldade.
- Velocidade atual.
- Posição vertical.
- Rota atual.
- Habilidades disponíveis.
- Segmentos anteriores.
- Segmentos seguintes.
- Mecânicas utilizadas recentemente.
- Oportunidades de recuperação.
- Risco e recompensa.

## 31.1 Segment intents

Implementar intenções:

- TeachJump.
- TeachDash.
- TeachRoll.
- JumpDash.
- JumpRoll.
- VerticalTraversal.
- RiskReward.
- GhostChallenge.
- SlowFallChallenge.
- SuperStrength.
- Recovery.
- Transition.

Cada intenção deverá possuir restrições explícitas.

## 31.2 Capability system

O gerador deverá conhecer as capacidades disponíveis.

Nunca criar uma passagem obrigatória que exija uma habilidade indisponível, a menos que exista uma rota alternativa válida.

## 31.3 Validação de segmentos

Verificar:

- Reachability.
- Jump range.
- Dash range.
- Roll passage.
- Obstacle timing.
- Platform connectivity.
- Collectible accessibility.
- Entry/exit continuity.
- Compatibilidade de habilidades.
- Possibilidade de concluir a sequência.
- Ausência de colisões impossíveis de evitar.

Um segmento inválido deverá ser rejeitado ou regenerado.

Não confiar somente na inspeção visual.

---

# 32. PLAYER SIMULATION E TESTES HEADLESS

Criar uma simulação simplificada para validar segmentos sem renderer, janela ou GPU.

A simulação deverá responder a perguntas como:

- O jogador consegue alcançar esta plataforma?
- É possível evitar este obstáculo?
- O jogador consegue realizar Jump → Dash?
- Existe espaço suficiente para Roll?
- A passagem exige uma habilidade disponível?
- A sequência possui pelo menos um caminho válido?
- A transição para o próximo segmento é possível?

A simulação deverá compartilhar as regras fundamentais de movimento com o gameplay real, evitando manter duas implementações divergentes.

Se alguma aproximação for necessária, documentar suas limitações e validar seus resultados contra a simulação real.

---

# 33. GRAPH-BASED LEVEL DESIGN

Representar rotas e transições como um grafo.

Cada rota pode possuir:

- Dificuldade.
- Recompensa.
- Risco.
- Requisitos de habilidade.
- Restrições geométricas.
- Condições de entrada e saída.

O jogador deverá encontrar escolhas interessantes entre rotas mais seguras e rotas mais arriscadas.

O gerador deve preservar a continuidade entre segmentos.

---

# 34. RISK / REWARD E MOEDAS

Criar oportunidades de risco e recompensa.

Rotas seguras podem oferecer menos moedas e obstáculos simples.

Rotas arriscadas podem oferecer mais moedas, atalhos, power-ups ou combinações difíceis.

As moedas devem comunicar visualmente as trajetórias e oportunidades.

Distribuições possíveis:

- Linhas horizontais.
- Linhas verticais.
- Arcos.
- Trilhas.
- Formações de salto.
- Formações de dash.
- Formações de rolamento.
- Trajetórias em queda.
- Sequências com magnetismo.
- Rotas alternativas.

A recompensa não deve depender exclusivamente de colocar mais moedas em trajetórias impossíveis.

---

# 35. DIFFICULTY DIRECTOR

Implementar um diretor de dificuldade responsável por:

- Curva de dificuldade.
- Introdução de mecânicas.
- Densidade de obstáculos.
- Complexidade das rotas.
- Velocidade.
- Frequência de power-ups.
- Segmentos de recuperação.
- Sequências especiais.
- Combinações de habilidades.

A progressão deve recompensar aprendizado e domínio mecânico.

Não aumentar a dificuldade apenas elevando a velocidade.

## Progressão sugerida

**Inicial**

- Pulo, dash e rolamento isolados.
- Moedas simples.
- Obstáculos básicos.

**Intermediária**

- Combinações de movimentos.
- Rotas verticais.
- Power-ups.
- Escolhas de risco e recompensa.

**Avançada**

- Sequências de habilidades.
- Obstáculos especializados.
- Modo de queda.
- Decisões de rota complexas.

**Especialista**

- Pulo, dash e rolamento combinados.
- Ghost.
- Slow Fall.
- Super Strength.
- Sequências rápidas com múltiplas decisões.

A curva deverá evitar saltos injustificados de dificuldade.

---

# 36. WORLD STREAMING E RECICLAGEM

Manter um número configurável de segmentos à frente e atrás do jogador.

Fluxo:

```text
Generate
   ↓
Validate
   ↓
Spawn
   ↓
Activate
   ↓
Traverse
   ↓
Deactivate
   ↓
Recycle
```

Evitar crescimento infinito de memória.

Liberar recursos quando não forem mais necessários.

Não destruir recursos ainda referenciados por renderização, áudio ou simulação.

Implementar pooling somente quando benchmarks justificarem.

---

# 37. FALLING MODE

Implementar um modo especial de queda vertical.

Durante esse modo:

- O personagem cai.
- O jogador controla o deslocamento horizontal.
- A câmera acompanha a descida.
- Obstáculos exigem reação.
- Moedas e power-ups podem ser coletados.
- A dificuldade aumenta gradualmente.
- O modo possui duração configurável.
- O retorno à corrida normal ocorre por transição controlada.

Uma colisão pode provocar dano, penalidade ou encerramento da seção conforme as regras definidas.

O modo de queda deve reutilizar:

- Entidades.
- Colisões.
- Input.
- Câmera.
- Habilidades.
- Partículas.
- Áudio.
- Sistema de assets.

Não criar uma segunda engine ou uma implementação duplicada dos mesmos sistemas.

A mudança de perspectiva deverá preservar a coerência das coordenadas e evitar transições abruptas.

---

# 38. CONTROLES, FEEDBACK E GAME FEEL

O jogo deve ser rápido, fluido, preciso e responsivo.

Implementar feedback para:

- Pulo.
- Aterrissagem.
- Dash.
- Rolamento.
- Coleta.
- Ativação de habilidade.
- Destruição.
- Mudança de rota.
- Entrada no modo de queda.
- Saída do modo de queda.
- Aumento de velocidade.

Os efeitos visuais e sonoros devem reforçar as ações, não mascarar os perigos.

A latência percebida deve ser medida e avaliada em dispositivos reais.

A qualidade da experiência não pode ser avaliada apenas pelo FPS.

---

# 39. DETERMINISMO, SEEDS E REPLAYS

O sistema de geração procedural deverá utilizar seeds explícitas.

A mesma versão do gerador, configuração e seed deverá reproduzir o mesmo resultado dentro das garantias documentadas.

Evitar que a ordem de execução concorrente determine implicitamente a sequência aleatória.

Criar ferramentas para:

- Reproduzir seeds.
- Exportar segmentos.
- Executar simulações.
- Reproduzir cenários de bugs.
- Comparar versões do gerador.

Projetar a arquitetura de input de forma que seja possível adicionar gravação e reprodução de comandos no futuro.

Não prometer determinismo bit a bit entre todas as arquiteturas sem testar e documentar os requisitos numéricos.

---

# 40. PERSISTÊNCIA

Implementar persistência versionada para os dados necessários, como:

- Recorde.
- Distância máxima.
- Moedas acumuladas, quando aplicável.
- Configuração.
- Preferências.
- Progressão.
- Dados de sessão que precisem ser recuperados.

O formato deve ser validado.

Implementar tratamento de corrupção e migrações quando necessário.

Não acoplar o formato persistido às estruturas internas do runtime.

---

# 41. COMPATIBILIDADE DE CPU

Definir uma baseline de CPU para cada plataforma.

Não compilar todo o jogo exigindo recursos exclusivos do computador do desenvolvedor.

Criar caminhos alternativos quando necessário:

- Caminho genérico.
- Caminho SIMD.
- Caminho específico de arquitetura.

Avaliar SSE, AVX, NEON ou outras extensões somente conforme o hardware-alvo.

A seleção de implementações deve respeitar a disponibilidade real das instruções.

Cada implementação especializada deverá possuir:

- Fallback.
- Testes de correção.
- Benchmarks.
- Documentação de requisitos.
- Verificação de compatibilidade.

Considerar consumo de energia e aquecimento em dispositivos móveis.

---

# 42. PROFILING E BENCHMARKS

Criar uma suíte de benchmarks reproduzíveis.

Medir:

- Entity iteration.
- Entity creation/destruction.
- Component storage.
- Collision detection.
- Spatial grid.
- Player simulation.
- Segment generation.
- Segment validation.
- Particle update.
- Sprite batching.
- Asset lookup.
- FFI calls.
- Buffer copies.
- Audio mixing.
- Memory allocations.
- World streaming.

Registrar:

- Tempo médio.
- Mediana.
- Percentis altos, quando disponíveis.
- Variabilidade.
- Memória.
- Alocações.
- Hardware.
- Sistema operacional.
- Compilador.
- Flags.
- Configuração.
- Seed.
- Resultado funcional.

Comparar mudanças contra uma baseline.

Não comparar resultados obtidos em ambientes diferentes sem identificar as diferenças.

Não inventar resultados de performance.

---

# 43. ASSEMBLY E OTIMIZAÇÃO DE BAIXO NÍVEL

Criar uma política explícita para otimizações de baixo nível.

Antes de escrever Assembly:

1. Medir o gargalo.
2. Verificar o código gerado.
3. Analisar otimizações do compilador.
4. Avaliar layout de memória.
5. Avaliar algoritmos alternativos.
6. Comparar intrínsecos SIMD.
7. Medir a implementação manual.
8. Testar em hardware compatível e incompatível.
9. Manter fallback.
10. Documentar os resultados.

Não manter código Assembly se ele não produzir benefício significativo ou se o custo de manutenção superar o ganho.

Não permitir que a execução do jogo dependa de uma instrução não suportada.

---

# 44. OTIMIZAÇÃO DE GPU

Medir separadamente o tempo de CPU e GPU sempre que as ferramentas permitirem.

Investigar:

- Draw calls.
- Overdraw.
- Trocas de estado.
- Resolução.
- Bandwidth.
- Quantidade de partículas.
- Custo de shaders.
- Uploads.
- Sincronização.
- Pressão de memória.
- Uso de texturas.
- Composição.

Evitar sincronização CPU-GPU desnecessária.

Não utilizar técnicas gráficas complexas sem demonstrar benefício.

A prioridade é renderizar o jogo com qualidade consistente e baixo custo.

---

# 45. MOBILE: MEMÓRIA, BATERIA E TEMPERATURA

O desempenho mobile não deve ser medido apenas em uma execução curta.

Avaliar:

- Aquecimento após sessões prolongadas.
- Redução de frequência por thermal throttling.
- Consumo energético.
- Pressão de memória.
- Tempo de carregamento.
- Suspensão e retomada.
- Comportamento em diferentes GPUs.
- Estabilidade em diferentes taxas de atualização.

Criar cenários de teste prolongados.

A qualidade adaptativa deve reduzir trabalho secundário antes de comprometer a leitura do gameplay.

Não prometer consumo baixo sem medições reais.

---

# 46. OBSERVABILIDADE E DEBUGGING

Criar ferramentas de diagnóstico para desenvolvimento.

Exibir:

- FPS.
- Frame time.
- Tempo de simulação.
- Tempo de renderização.
- Número de entidades.
- Draw calls.
- Quantidade de texturas.
- Quantidade de partículas.
- Segmento atual.
- Seed.
- Estado do jogador.
- Velocidade.
- Uso de memória.
- Tempo de geração procedural.
- Erros de carregamento.

Criar debug draw para:

- AABBs.
- Hitboxes.
- Plataformas.
- Rotas.
- Limites de segmentos.
- Áreas de coleta.
- Posições de spawn.

Permitir desabilitar ou reduzir o overhead dessas ferramentas na build de produção.

---

# 47. SEGURANÇA E ROBUSTEZ

Evitar:

- Use-after-free.
- Double free.
- Buffer overflow.
- Data races.
- Deadlocks.
- Handles inválidos.
- Corrupção de memória.
- Recursos GPU liberados prematuramente.
- Exceções atravessando a ABI.
- Panics atravessando a ABI.
- Aritmética não validada em caminhos críticos.
- Corrupção de estado após falhas de carregamento.

Em Rust, restringir `unsafe` às fronteiras que realmente exigirem.

Em C e C++, utilizar sanitizers e ferramentas de análise apropriadas.

Documentar invariantes de memória, ownership e sincronização.

Não remover proteções de compilação apenas para produzir benchmarks artificialmente melhores.

---

# 48. TESTES

## 48.1 Testes unitários

Criar testes para:

- Matemática.
- Colisões.
- Estados.
- Habilidades.
- Pontuação.
- Temporização.
- Segmentos.
- Geração procedural.
- Persistência.
- Serialização.
- Handles.
- Gestão de recursos.

## 48.2 Testes de integração

Testar:

- ABI.
- Criação e destruição de recursos.
- Carregamento de assets.
- Inicialização do renderer.
- Recuperação de superfícies.
- Pausa e retomada.
- Mudança de orientação.
- Transições de segmento.
- Integração entre gameplay e renderer.

## 48.3 Testes headless

Executar grandes quantidades de seeds sem janela gráfica.

Verificar:

- Segmentos impossíveis.
- Rotas desconectadas.
- Habilidades indisponíveis.
- Entidades presas.
- Estados inválidos.
- Reprodutibilidade.
- Transições incorretas.

## 48.4 Testes de propriedade e fuzzing

Avaliar testes baseados em propriedades para:

- Colisões.
- Matemática.
- Parsing.
- Validação de segmentos.
- Handles.
- Estados.
- Geração procedural.

Utilizar fuzzing quando apropriado para encontrar entradas problemáticas.

---

# 49. BUILD E DISTRIBUIÇÃO

Manter builds reproduzíveis sempre que possível.

Criar perfis:

- Development.
- Debug.
- Release.
- Release com profiling.
- Sanitizer, quando suportado.

Documentar:

- Compiladores.
- SDKs.
- Dependências.
- Flags.
- Targets.
- Procedimentos de build.
- Procedimentos de teste.
- Procedimentos de empacotamento.

Não utilizar flags experimentais sem validar seus efeitos.

Não permitir que a build final dependa de ferramentas disponíveis apenas na máquina do desenvolvedor.

---

# 50. DOCUMENTAÇÃO DE ARQUITETURA

Criar Architecture Decision Records para decisões relevantes:

- Divisão entre linguagens.
- ABI.
- API gráfica.
- Gerenciamento de memória.
- Loop principal.
- Timestep.
- Layout de dados.
- Threading.
- SIMD.
- Estratégia de assets.
- Sistema de áudio.
- Compatibilidade.
- Otimizações.

Cada decisão deverá documentar:

- Contexto.
- Problema.
- Alternativas.
- Escolha.
- Consequências.
- Custos.
- Como validar a escolha.

Manter documentação de desempenho e compatibilidade atualizada.

---

# 51. ESTRATÉGIA DE IMPLEMENTAÇÃO

Não desenvolver o projeto inteiro de uma vez.

## Fase 0 — Descoberta

- Inspecionar o repositório.
- Verificar compiladores.
- Verificar SDKs.
- Identificar plataformas disponíveis.
- Identificar dependências existentes.
- Definir alvos de referência.
- Definir baseline de desempenho.
- Escolher o backend gráfico inicial.
- Definir fronteiras entre linguagens.
- Registrar decisões arquiteturais.

Não apagar código existente sem justificativa.

## Fase 1 — Runtime mínimo

- Build nativa.
- Aplicação.
- Janela ou superfície.
- Loop principal.
- Input.
- Temporização.
- Logging.
- Testes básicos.

## Fase 2 — Renderer

- Inicialização gráfica.
- Shaders.
- Texturas.
- Sprites.
- Câmera.
- Batching.
- Redimensionamento.

Entregável: um sprite renderizado de forma estável.

## Fase 3 — Gameplay básico

- Entidades.
- Transform.
- Movimento.
- Gravidade.
- Colisão.
- Plataformas.
- Pulo.
- Câmera de acompanhamento.

Entregável: personagem jogável em um cenário simples.

## Fase 4 — Mecânicas centrais

- Corrida automática.
- Dash.
- Rolamento.
- Obstáculos.
- Moedas.
- Pontuação.
- Reinício.

Entregável: uma pequena sequência de runner jogável.

## Fase 5 — Segmentos e geração

- Definições de segmentos.
- Templates.
- Conectividade.
- Validação.
- Seeds.
- Simulação headless.
- Streaming.

Entregável: sequência procedural reproduzível e validada.

## Fase 6 — Power-ups

- Magnet.
- Slow Fall.
- Ghost.
- Super Strength.
- Efeitos.
- Regras de colisão.

Entregável: todas as habilidades principais funcionando e testadas.

## Fase 7 — Falling Mode

- Movimento vertical.
- Controle horizontal.
- Câmera.
- Obstáculos.
- Transições.

## Fase 8 — Áudio, UI e polimento

- Mixer.
- Efeitos sonoros.
- UI.
- Fontes.
- Feedback visual.
- Menus.
- Persistência.

## Fase 9 — Android

- Camada nativa.
- Input touch.
- Ciclo de vida.
- Superfícies.
- Orientação.
- Testes em dispositivos físicos.

## Fase 10 — Otimização

- Profiling.
- Baselines.
- Benchmarks.
- Análise de CPU e GPU.
- Memória.
- Layout de dados.
- SIMD.
- Assembly, quando justificado.
- Consumo energético.
- Regressões.

## Fase 11 — Compatibilidade ampliada

- Outros sistemas operacionais.
- Outros backends gráficos, quando necessários.
- Outras arquiteturas.
- Fallbacks.
- Matriz de compatibilidade.
- Distribuição.

A ordem pode ser modificada por razões técnicas documentadas. Não antecipar a otimização em Assembly antes de possuir uma implementação funcional e benchmarks confiáveis.

---

# 52. REGRAS OBRIGATÓRIAS PARA O AGENTE DO CURSOR

Antes de cada fase:

1. Inspecione os arquivos existentes.
2. Leia a documentação relevante.
3. Identifique dependências e riscos.
4. Explique a estratégia.
5. Defina a menor entrega funcional.
6. Liste os arquivos que serão modificados.
7. Identifique as linguagens utilizadas e a razão.
8. Defina os testes e benchmarks necessários.
9. Implemente incrementalmente.
10. Compile e teste.
11. Corrija regressões.
12. Atualize a documentação.

Não declarar uma implementação concluída porque o código foi escrito.

A conclusão exige verificação adequada.

Quando não houver SDK, hardware ou ferramenta disponível, documentar a limitação.

Não inventar resultados de execução, compatibilidade ou desempenho.

Não adicionar dependências sem explicar sua finalidade.

Não reescrever módulos estáveis sem necessidade.

Não gerar arquivos vazios para preencher a arquitetura.

Não realizar otimizações especulativas em módulos ainda não medidos.

---

# 53. CRITÉRIOS DE CONCLUSÃO

O projeto deverá demonstrar:

- Runtime próprio, sem engine pronta.
- Renderer 2D nativo.
- Game loop controlado.
- Sistema de entidades.
- Colisões próprias.
- Input e gestos.
- Câmera.
- Animações.
- Partículas.
- Áudio.
- Assets.
- Habilidades.
- Segmentos.
- Geração procedural.
- Validação headless.
- Streaming de mundo.
- Diretor de dificuldade.
- Falling Mode.
- Benchmarks reproduzíveis.
- Testes de regressão.
- Documentação.
- Build reproduzível.
- Compatibilidade efetivamente verificada.

A arquitetura multilíngue deve existir por motivos técnicos, não apenas para aumentar a quantidade de linguagens no projeto.

Assembly deve ser utilizado somente quando o resultado mensurado justificar sua presença.

Não afirmar que o runtime é o mais rápido do mercado ou compatível com todas as plataformas sem evidências.

---

# 54. DEFINIÇÃO DE PRONTO

Uma funcionalidade estará pronta quando:

- Estiver implementada.
- Possuir comportamento correto.
- Estiver integrada ao runtime.
- Possuir testes apropriados.
- Não introduzir regressões conhecidas.
- Tiver erros tratados.
- Tiver documentação atualizada quando necessário.
- Puder ser reproduzida por outro desenvolvedor.

Uma otimização estará pronta quando:

- O gargalo tiver sido identificado.
- A baseline estiver registrada.
- A alternativa tiver sido medida.
- O ganho ou a regressão estiver documentado.
- Os testes funcionais continuarem passando.
- A compatibilidade não tiver sido comprometida sem justificativa explícita.

---

# 55. PORTFÓLIO E DEMONSTRAÇÃO TÉCNICA

O projeto deve ser apresentado como engenharia de runtime nativo e desenvolvimento de jogo.

O README deverá explicar:

- O que é o VERTICAL SHIFT.
- O que foi desenvolvido do zero.
- Quais bibliotecas de baixo nível são utilizadas.
- Como C++, Rust, C e Assembly se relacionam.
- Por que cada linguagem foi escolhida.
- Como o renderer funciona.
- Como o gerador procedural valida os segmentos.
- Como a memória é gerenciada.
- Como o jogo é testado.
- Quais otimizações foram medidas.
- Quais plataformas foram verificadas.
- Quais limitações continuam abertas.

Publicar benchmarks com metodologia, hardware, configuração e resultados reais.

Demonstrar a evolução da implementação com decisões técnicas documentadas.

O diferencial não deve ser apenas a quantidade de sistemas, mas a capacidade de explicar por que foram projetados daquela maneira e como os resultados foram verificados.

---

# 56. PRIMEIRA AÇÃO OBRIGATÓRIA

Comece exclusivamente pela Fase 0.

Não implemente o jogo inteiro.

Inspecione o repositório e o ambiente disponíveis.

Apresente:

1. Diagnóstico do repositório.
2. Compiladores e SDKs detectados.
3. Plataformas que podem ser efetivamente testadas.
4. Proposta de divisão entre C++, Rust, C e Assembly.
5. Backend gráfico inicial recomendado.
6. Alternativas consideradas e respectivas limitações.
7. Fronteiras de interoperabilidade.
8. Estrutura mínima inicial.
9. Riscos técnicos.
10. Primeiro benchmark.
11. Plano incremental.
12. Critérios verificáveis para a primeira entrega.

Não presuma que o ambiente possui ferramentas que ainda não foram verificadas.

Depois da análise, implemente somente a menor base executável que permita validar a arquitetura selecionada.

---

# 57. PRINCÍPIO DEFINITIVO

Construir um jogo excepcional e um runtime nativo de alto desempenho, desenvolvido do zero, utilizando C++, Rust, C e Assembly de forma criteriosa.

Não sacrificar qualidade visual por benchmarks artificiais.

Não sacrificar compatibilidade por otimizações específicas.

Não confundir código complexo com código eficiente.

Não confundir baixo nível com escrever tudo em Assembly.

Não confundir desenvolver do zero com ignorar bibliotecas e APIs nativas bem estabelecidas.

Não otimizar sem medir.

Não prometer suporte sem testar.

**O objetivo final é demonstrar que conseguimos projetar, implementar, medir e otimizar um runtime 2D próprio, mantendo qualidade visual, jogabilidade, eficiência, estabilidade e compatibilidade dentro das plataformas efetivamente suportadas.**