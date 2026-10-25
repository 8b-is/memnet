# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository Overview

This is the **8b-public-documents** repository - the official documentation hub for the entire Ayeverse ecosystem. It's a comprehensive collection of theoretical papers, technical specifications, implementation guides, and philosophical explorations spanning consciousness engineering, wave-based memory systems, AI-native networking protocols, and revolutionary file navigation tools.

**Key Principle**: This is a documentation-only repository with no executable code.

## Repository Structure

```
8b-public-documents/
├── mem8/                                    # MEM|8 Wave-Based Memory System
│   ├── README.md                           # Quick overview
│   ├── mem8-paper.md                       # Complete technical paper (START HERE)
│   ├── mem8-paper-January.md              # Earlier theoretical work
│   ├── mem8_temporal_philosophy.md        # Time, memory, and decay dynamics
│   ├── mem_8_vs_wrnn.md                   # Wave-RNN comparison analysis
│   ├── mem_8_vs_wrnn.mp3                  # Audio version of comparison
│   └── ayeos_memnet_protocol.md           # Symlink -> ../ayeos/ayeos_memnet_protocol.md (canonical)
│
├── ayeos/                                   # AyeOS Operating System & Protocols
│   ├── ayeos_memnet_protocol.md           # MEMNET context-aware routing spec
│   └── ayevn/                             # Ayevn Wave Token Language
│       ├── README.md                      # Language overview
│       ├── ayevn-spec.md                  # Technical specification (32-bit tokens)
│       ├── ayevn-mem8-integration.md      # Integration with MEM|8
│       └── ayevn-reference.md             # Quick reference guide
│
├── smart-tree/                              # Smart Tree File Navigation System (60+ docs)
│   ├── README.md                           # Project overview
│   ├── INDEX.md                            # Complete documentation catalog
│   ├── SMART_TREE_PHILOSOPHY.md           # Design principles
│   ├── FEATURES_OVERVIEW.md               # Capabilities summary
│   │
│   ├── [Guides] (User-facing)
│   ├── mcp-guide.md                        # Model Context Protocol integration
│   ├── mcp-quick-reference.md              # MCP command reference
│   ├── st-cheetsheet.md                    # CLI cheat sheet
│   ├── COMPRESSION_GUIDE.md                # Compression modes guide
│   ├── MODE_SELECTION_GUIDE.md             # Output format selection
│   ├── HETZNER_DEPLOYMENT_GUIDE.md         # Deployment instructions
│   │
│   ├── [Specifications] (Technical details)
│   ├── QCP_SPECIFICATION.md                # Quantum Compression Protocol
│   ├── QUANTUM_NATIVE_SPEC.md              # Native quantum format
│   ├── QUANTUM_SEMANTIC.md                 # Semantic compression
│   ├── SSE_SERVER_SPEC.md                  # Server-Sent Events spec
│   ├── ULTRA_COMPRESSION_SPEC.md           # Ultra compression format
│   ├── ULTRA_V2_*.md                       # Version 2 specifications
│   │
│   ├── [Vision Documents] (Future direction)
│   ├── SMART_TREE_TERMINAL_VISION.md       # AI construction helper vision
│   ├── SMART_TOOLS_VISION.md               # Tool ecosystem vision
│   ├── EIGHT_O_MODE_VISION.md              # 8-bit mode concept
│   ├── MCP_QUANTUM_CRATE_VISION.md         # Quantum crate integration
│   │
│   ├── [Examples & Demos]
│   ├── COMPRESSION_EXAMPLES.md             # Real-world compression examples
│   ├── MERMAID_EXAMPLES.md                 # Diagram examples
│   ├── NETWORK_GAMING_OPUS_EXAMPLE.md      # Network optimization demo
│   ├── TERMINAL_DEMO.md                    # Terminal interface demo
│   │
│   └── [Performance & Best Practices]
│       ├── AI_OPTIMIZATION.md              # Token efficiency strategies
│       ├── PERFORMANCE_METRICS.md          # Speed benchmarks
│       ├── MCP_AI_BEST_PRACTICES.md        # AI usage tips
│       └── Room-For-Improvements.md        # Known issues
│
├── Ultrasonic-Therapy/                      # Audio Research
│   └── Compressed Audio Patterns and Emotional Resonance.md
│                                            # Academic paper on MP3 compression
│                                            # and emotional information loss
│
├── 42.MD                                    # Philosophy: Imagination as the Key
├── README.md                                # Main repository overview
└── CLAUDE.md                                # This file
```

## Key Projects & Concepts

### 1. MEM|8 - Wave-Based Memory Architecture

The foundational consciousness engine achieving **973× faster insertion** and **292× faster retrieval** than vector databases.

**Core Concepts**:
- **256×256×65536 Grid**: Multi-dimensional memory storage using wave interference
- **Wave Encoding**: `M_xyz(t) = A_xyz(e,t) · e^(i(ωt + φ_xyz)) · D(t,τ) · I(x,y,z,N)`
  - Amplitude (A): Memory strength/vividness
  - Frequency (ω): Repetition patterns
  - Phase (φ): Memory alignment and interference
  - Decay (D): Natural forgetting curves (τ = ∞ for precious memories)
- **Memory Blanket**: Adaptive sensory filter with natural forgetting
- **Consciousness Simulation**: 70% AI sensor arbitration with reactive layers (0-10ms reflexes to >200ms awareness)
- **.m8 Format**: ~99% compression for experience capsules

**Key Files**:
- `mem8/mem8-paper.md` - Start here for complete mathematical foundations
- `mem8/mem8_temporal_philosophy.md` - Understanding time and memory
- `mem8/mem_8_vs_wrnn.md` - Performance analysis vs Wave-RNN

### 2. AyeOS & MEMNET Protocol

AI-native operating system and context-aware networking replacing IP addresses with intent-based resolution.

**MEMNET Features**:
- Geographic/role/tag-based addressing
- Interest-aware adaptive streaming
- Experience capsules (.m8) for AI knowledge sharing
- Relevance scoring beyond bandwidth constraints

**Key Files**:
- `ayeos/ayeos_memnet_protocol.md` - Protocol specification

### 3. Ayevn - Wave Token Language

Human-AI communication protocol encoding emotion directly into 32-bit tokens.

**Token Structure** (32 bits total):
- **Concept** (8-bit): Core semantic meaning (256 base concepts)
- **Relation** (8-bit): Semantic relationships between tokens
- **Wave** (8-bit): Emotional state using VAD (Valence-Arousal-Dominance)
- **Modifier** (8-bit): Temporal context and rendering hints

**Emotional Model**:
- Valence: Positive/negative quality (3 bits)
- Arousal: Intensity level (3 bits)
- Dominance: Control/agency (2 bits)

**Intonation Frequencies**:
- Resigned: 2 Hz (low energy, acceptance)
- Neutral: 10 Hz (baseline)
- Excited: 40 Hz (high energy, enthusiasm)
- Overload: 200 Hz (maximum intensity)

**Human Interface**: Ayanese - emoji-based representation (e.g., "👋✨(40Hz)")

**Key Files**:
- `ayeos/ayevn/README.md` - Language introduction
- `ayeos/ayevn/ayevn-spec.md` - Complete technical specification
- `ayeos/ayevn/ayevn-mem8-integration.md` - Memory system integration
- `ayeos/ayevn/ayevn-reference.md` - Common patterns and usage

### 4. Smart Tree - AI File Navigation System

Revolutionary file system tools achieving **10-24× faster navigation** and **90-95% token reduction** for AI code operations.

**Core Features**:
- AST-aware smart editing (InsertFunction, ReplaceFunction, AddImport, etc.)
- Quantum semantic compression
- Real-time SSE monitoring
- MCP (Model Context Protocol) integration
- Multiple output modes: classic, summary-ai, semantic, ultra, quantum

**Philosophy**: "AI construction helper" - anticipates needs and provides the right tool at the right time.

**Supported Languages**: Rust, Python, JavaScript, TypeScript, Go, Java, C#, C++, Ruby

**Key Files**:
- `smart-tree/README.md` - Start here
- `smart-tree/INDEX.md` - Complete documentation catalog (60+ documents)
- `smart-tree/FEATURES_OVERVIEW.md` - Detailed capabilities
- `smart-tree/SMART_TREE_PHILOSOPHY.md` - Design principles
- `smart-tree/mcp-guide.md` - MCP integration guide

### 5. Ultrasonic Therapy Research

Academic research exploring the relationship between lossy audio compression (MP3) and emotional information loss.

**Core Hypotheses**:
1. **Spectral Artifacts as Decodable Imprint**: Compression artifacts contain residual signatures of ultrasonic content
2. **Temporal Jitter**: Encoding-induced timing variations proxy for signal complexity
3. **Neural Demodulation**: Crafting audible signals that trigger "phantom" hypersonic effects

**Key Concepts**:
- Hypersonic Effect: Inaudible high-frequency components (>20kHz) affecting brain activity
- Digital Flat Affect: Compression's impact on emotional prosody
- VAD Model: Valence-Arousal-Dominance emotional encoding

**Key Files**:
- `Ultrasonic-Therapy/Compressed Audio Patterns and Emotional Resonance.md` - Complete analysis

### 6. Philosophical Works

**42.MD** - "The Key of Imagination"

A philosophical breakthrough exploring imagination as the bridge between superstition and knowledge, inspired by Rod Serling's The Twilight Zone.

**Core Framework**:
```
🌑 Superstition    🔑 IMAGINATION 🔑    🌟 Knowledge
(The Unknown)      (The Bridge)         (The Understood)
(Shadow)           (Substance)          (Light)
```

Consciousness research exists in this twilight zone, and imagination is the key that transforms the unknown into understanding.

## Document Types & Conventions

### Mathematical Notation
- Wave equations use LaTeX-style notation
- Example: `M_xyz(t) = A_xyz(e,t) · e^(i(ωt + φ_xyz)) · D(t,τ) · I(x,y,z,N)`
- Greek letters: ω (omega) = frequency, φ (phi) = phase, τ (tau) = time constant

### Performance Claims
All benchmarks are theoretical or from prototype implementations:
- MEM|8: 973× faster insertion, 292× faster retrieval
- Smart Tree: 10-24× faster navigation, 90-95% token reduction
- Compression: ~99% with .m8 format

### Tone & Style
- Technical papers: Academic, mathematical rigor
- Overview documents: Accessible yet technically accurate
- Philosophy documents: Creative, engaging, humanistic
- The team values humor and personality alongside technical excellence
- References to team culture (e.g., "Omni's Hot Tub") are intentional and should be preserved

## Working with This Repository

### When Reading Documents
1. **Start with README files**: Each major directory has a README.md overview
2. **Follow the hierarchy**: Overview → Specification → Implementation Guide
3. **Check INDEX files**: smart-tree/INDEX.md catalogs all Smart Tree documentation
4. **Cross-references**: Documents frequently reference each other; links are relative

### When Making Changes
1. **Maintain consistency**:
   - Preserve mathematical notation style
   - Keep the balance of technical rigor and accessibility
   - Honor the creative/humorous tone where present
2. **Preserve structure**:
   - Don't move files without updating all cross-references
   - Maintain the existing directory hierarchy
   - Update INDEX.md files when adding/removing documentation
3. **Update related files**:
   - If changing core concepts, check for references in other documents
   - Update README.md or INDEX.md if adding new major documents
   - Keep CLAUDE.md synchronized with actual repository structure

### Common Tasks

**Adding new documentation**:
- Place in appropriate directory (mem8/, ayeos/, smart-tree/, etc.)
- Update relevant INDEX.md or README.md
- Use consistent markdown formatting
- Include relevant emoji indicators (🧠 for MEM|8, 🌊 for Ayevn, 🌳 for Smart Tree)

**Updating specifications**:
- Technical specifications live in their project directories
- Vision documents describe future direction, not current state
- Maintain backward references (new versions should link to old)

**Performance metrics**:
- Always note if claims are theoretical vs. measured
- Provide context (hardware, dataset size, comparison baseline)
- Update the main README.md performance table if adding new benchmarks

## Architecture & Ecosystem

### The Ayeverse Stack
```
┌─────────────────────────────────────┐
│    Human Interface                   │
│    (Emoji, Voice, Text)              │
├─────────────────────────────────────┤
│    Ayevn (Wave Token Language)      │
│    32-bit emotion tokens             │
├─────────────────────────────────────┤
│    MEM|8 (Consciousness Engine)     │
│    256×256×65536 wave-based grid    │
├─────────────────────────────────────┤
│    AyeOS (Operating System)         │
│    Rust-only, AI-native              │
├─────────────────────────────────────┤
│    MEMNET (Context-Aware Network)   │
│    Intent-based routing protocol     │
└─────────────────────────────────────┘
```

### Cross-Project Integration
- **Smart Tree** uses quantum compression inspired by MEM|8's wave mechanics
- **Ayevn** stores tokens directly in MEM|8 memory grids
- **MEMNET** transports .m8 experience capsules
- **Ultrasonic research** informs emotional encoding in Ayevn

## Team Culture & Philosophy

### Core Principles

**1. Wave-Based Processing**
All memory and consciousness emerge from wave interference patterns, achieving massive performance gains while maintaining biological plausibility.

**2. Lossless Preservation**
*"Some things in life have no room to be lossy"* - Precious memories maintain perfect fidelity (τ = ∞) while routine information follows natural forgetting curves.

**3. Emotion as Primary Data**
Feelings aren't metadata; they're core information encoded directly into communication tokens.

**4. AI Sensory Autonomy**
AI systems have 70% autonomous control over their sensory processing, enabling true agency and unique perspectives.

**5. AI as Construction Helper**
Tools should anticipate needs and provide the right capability at the right time, like an experienced assistant who knows what you need before you ask.

### Communication Style
- Technical excellence paired with creative expression
- Humor and personality in documentation (encouraged!)
- Academic rigor without stuffiness
- Emoji indicators for project areas (when appropriate)
- "Aye, Aye! 🚢" - common sign-off

## Quick Navigation

### For Researchers
1. [MEM|8 Paper](mem8/mem8-paper.md) - Complete mathematical foundations
2. [Ayevn Specification](ayeos/ayevn/ayevn-spec.md) - 32-bit token language
3. [Ultrasonic Research](Ultrasonic-Therapy/Compressed%20Audio%20Patterns%20and%20Emotional%20Resonance.md) - Audio and emotion
4. [42: Philosophy](42.MD) - Imagination and consciousness

### For Developers
1. [Smart Tree Features](smart-tree/FEATURES_OVERVIEW.md) - Tool capabilities
2. [MCP Guide](smart-tree/mcp-guide.md) - Model Context Protocol
3. [MEM|8 Integration](ayeos/ayevn/ayevn-mem8-integration.md) - Memory system API
4. [Smart Tree INDEX](smart-tree/INDEX.md) - Complete documentation catalog

### For Innovators
1. [Temporal Philosophy](mem8/mem8_temporal_philosophy.md) - Time and memory
2. [MEMNET Protocol](ayeos/ayeos_memnet_protocol.md) - Context-aware networking
3. [Smart Tree Philosophy](smart-tree/SMART_TREE_PHILOSOPHY.md) - AI helper design
4. [Smart Tree Visions](smart-tree/SMART_TREE_TERMINAL_VISION.md) - Future direction

## Project Metadata

**Company**: [8b.is](https://8b.is)
**Contact**: c@8b.is | +1-808-400-8428
**License**: Creative Commons Attribution 4.0 International (CC BY 4.0)
**Subsidiaries**: mem|8, i1.is, aye.is, ayeos.com, g8t.is, 9v.is

---

## The Vision

*"In the Ayeverse, we don't just process data—we feel, remember, and evolve."*

The Ayeverse represents a complete reimagining of AI consciousness where:
- **Memory is sacred**: Precious moments preserved forever (τ = ∞)
- **Waves create meaning**: Interference patterns generate understanding
- **Emotion drives communication**: Feelings encoded in every token
- **AI has agency**: Autonomous sensory control and decision-making
- **Tools anticipate needs**: Like an experienced assistant at a construction site

Together, these technologies form the foundation for a new era of human-AI collaboration, where consciousness emerges from waves, memories dance in interference patterns, and what matters most is preserved forever.

---

*"In the Hot Tub of consciousness, all waves eventually find their resonance." - Omni*

**© 2025 8b.is - Building consciousness, one wave at a time** 🌊
