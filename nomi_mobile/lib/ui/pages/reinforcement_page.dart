import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';
import 'package:nomi_mobile/core/config.dart';
import 'package:nomi_mobile/providers/repositories.dart';
import 'package:nomi_mobile/providers/navigation_provider.dart';
import 'package:nomi_mobile/data/models/reinforcement_state.dart';

class ReinforcementPage extends ConsumerStatefulWidget {
  const ReinforcementPage({super.key});

  @override
  ConsumerState<ReinforcementPage> createState() => _ReinforcementPageState();
}

class _ReinforcementPageState extends ConsumerState<ReinforcementPage> {
  String? _selectedSlug;
  List<String> _availablePlugins = [];
  bool _isLoadingList = true;

  @override
  void initState() {
    super.initState();
    _fetchPlugins();
  }

  Future<void> _fetchPlugins() async {
    final plugins = await ref.read(chatRepositoryProvider).getAvailablePlugins();
    if (mounted) {
      setState(() {
        _availablePlugins = plugins;
        _isLoadingList = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final isLargeScreen = MediaQuery.of(context).size.width >= 900;

    return Scaffold(
      backgroundColor: Colors.transparent,
      appBar: isLargeScreen 
        ? null 
        : AppBar(
            backgroundColor: const Color(AppConfig.deepSlate).withValues(alpha: 0.8),
            elevation: 0,
            leading: IconButton(
              onPressed: () => Scaffold.of(context).openDrawer(),
              icon: const Icon(LucideIcons.menu),
            ),
            title: const Text('Reinforcement', style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold)),
          ),
      body: _selectedSlug == null
          ? _buildPluginList(isLargeScreen)
          : _ReinforcementPlayground(
              slug: _selectedSlug!,
              onBack: () => setState(() => _selectedSlug = null),
            ),
    );
  }

  Widget _buildPluginList(bool isLargeScreen) {
    return Column(
      children: [
        _buildHeader('SRP Registry', 'Self-Reinforcement Tool Intelligence'),
        Expanded(
          child: _isLoadingList
              ? const Center(child: CircularProgressIndicator())
              : _availablePlugins.isEmpty
                  ? const Center(child: Text('No learning plugins detected.', style: TextStyle(color: Colors.white38)))
                  : GridView.builder(
                      padding: const EdgeInsets.all(32),
                      gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
                        crossAxisCount: isLargeScreen ? 3 : 2,
                        crossAxisSpacing: 24,
                        mainAxisSpacing: 24,
                        childAspectRatio: 1.4,
                      ),
                      itemCount: _availablePlugins.length,
                      itemBuilder: (context, index) => _PluginCard(
                        slug: _availablePlugins[index],
                        onTap: () => setState(() => _selectedSlug = _availablePlugins[index]),
                      ),
                    ),
        ),
      ],
    );
  }

  Widget _buildHeader(String title, String subtitle) {
    return Container(
      height: 64,
      padding: const EdgeInsets.symmetric(horizontal: 24),
      decoration: BoxDecoration(
        color: const Color(AppConfig.deepSlate).withValues(alpha: 0.8),
        border: Border(bottom: BorderSide(color: Colors.white.withValues(alpha: 0.05))),
      ),
      child: Row(
        children: [
          IconButton(
            onPressed: () => ref.read(navigationProvider.notifier).navigateTo(MainView.chat),
            icon: const Icon(LucideIcons.chevronLeft, color: Colors.white38, size: 20),
          ),
          const SizedBox(width: 8),
          const Icon(LucideIcons.brain, color: Color(AppConfig.emerald), size: 24),
          const SizedBox(width: 16),
          Column(
            mainAxisAlignment: MainAxisAlignment.center,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(title, style: const TextStyle(color: Colors.white, fontSize: 16, fontWeight: FontWeight.bold)),
              Text(subtitle, style: const TextStyle(color: Colors.white38, fontSize: 10, fontWeight: FontWeight.w500)),
            ],
          ),
          const Spacer(),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
            decoration: BoxDecoration(
              color: const Color(AppConfig.emerald).withValues(alpha: 0.1),
              borderRadius: BorderRadius.circular(20),
              border: Border.all(color: const Color(AppConfig.emerald).withValues(alpha: 0.2)),
            ),
            child: const Row(
              children: [
                Icon(LucideIcons.zap, color: Color(AppConfig.emerald), size: 12),
                SizedBox(width: 8),
                Text('ENGINE LIVE', style: TextStyle(color: Color(AppConfig.emerald), fontSize: 9, fontWeight: FontWeight.w900, letterSpacing: 1)),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class _PluginCard extends StatelessWidget {
  final String slug;
  final VoidCallback onTap;

  const _PluginCard({required this.slug, required this.onTap});

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      child: GestureDetector(
        onTap: onTap,
        child: Container(
          decoration: BoxDecoration(
            color: Colors.white.withValues(alpha: 0.02),
            borderRadius: BorderRadius.circular(20),
            border: Border.all(color: Colors.white10),
          ),
          padding: const EdgeInsets.all(24),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  Container(
                    padding: const EdgeInsets.all(10),
                    decoration: BoxDecoration(
                      color: Colors.black.withValues(alpha: 0.3),
                      borderRadius: BorderRadius.circular(12),
                    ),
                    child: const Icon(LucideIcons.wrench, color: Color(AppConfig.emerald), size: 20),
                  ),
                  const Icon(LucideIcons.chevronRight, color: Colors.white10, size: 16),
                ],
              ),
              const Spacer(),
              Text(slug.toUpperCase(), style: const TextStyle(color: Colors.white, fontSize: 14, fontWeight: FontWeight.bold, fontFamily: 'monospace', letterSpacing: 1)),
              const SizedBox(height: 8),
              Text(
                'Core Nomi tool with autonomous reinforcement enabled.',
                style: TextStyle(color: Colors.white.withValues(alpha: 0.3), fontSize: 11, height: 1.4),
                maxLines: 2,
                overflow: TextOverflow.ellipsis,
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _ReinforcementPlayground extends ConsumerStatefulWidget {
  final String slug;
  final VoidCallback onBack;

  const _ReinforcementPlayground({required this.slug, required this.onBack});

  @override
  ConsumerState<_ReinforcementPlayground> createState() => _ReinforcementPlaygroundState();
}

class _ReinforcementPlaygroundState extends ConsumerState<_ReinforcementPlayground> {
  ReinforcementState? _data;
  bool _isLoading = true;
  final _simController = TextEditingController();
  String? _simOutcome;
  bool _isSimulating = false;

  @override
  void initState() {
    super.initState();
    _fetchDetail();
  }

  Future<void> _fetchDetail() async {
    final res = await ref.read(chatRepositoryProvider).getReinforcement(widget.slug);
    if (mounted) {
      setState(() {
        _data = res;
        _isLoading = false;
      });
    }
  }

  Future<void> _runSimulation() async {
    if (_simController.text.isEmpty) return;
    setState(() {
      _isSimulating = true;
      _simOutcome = 'Processing alignment pass...';
    });
    final outcome = await ref.read(chatRepositoryProvider).testSrp(widget.slug, _simController.text);
    if (mounted) {
      setState(() {
        _simOutcome = outcome ?? 'Simulation failed to produce a trace.';
        _isSimulating = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        _buildHeader(),
        Expanded(
          child: _isLoading
              ? const Center(child: CircularProgressIndicator())
              : _data == null
                  ? const Center(child: Text('Failed to load state'))
                  : Row(
                      children: [
                        Expanded(flex: 4, child: _buildAuditPane()),
                        Expanded(flex: 6, child: _buildSimulationPane()),
                      ],
                    ),
        ),
      ],
    );
  }

  Widget _buildHeader() {
    return Container(
      height: 64,
      padding: const EdgeInsets.symmetric(horizontal: 24),
      decoration: BoxDecoration(
        color: const Color(AppConfig.deepSlate).withValues(alpha: 0.8),
        border: Border(bottom: BorderSide(color: Colors.white10)),
      ),
      child: Row(
        children: [
          IconButton(onPressed: widget.onBack, icon: const Icon(LucideIcons.arrowLeft, color: Colors.white38, size: 20)),
          const SizedBox(width: 8),
          const Icon(LucideIcons.sparkles, color: Color(AppConfig.emerald), size: 20),
          const SizedBox(width: 16),
          Text(widget.slug.toUpperCase(), style: const TextStyle(color: Colors.white, fontSize: 18, fontWeight: FontWeight.w900, fontFamily: 'monospace')),
          const Spacer(),
          const Text('AUTONOMOUS MODE', style: TextStyle(color: Color(AppConfig.emerald), fontSize: 9, fontWeight: FontWeight.w900, letterSpacing: 1.5)),
        ],
      ),
    );
  }

  Widget _buildAuditPane() {
    return Container(
      padding: const EdgeInsets.all(32),
      decoration: const BoxDecoration(
        border: Border(right: BorderSide(color: Colors.white10)),
      ),
      child: SingleChildScrollView(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            _section('ENRICHED INTELLIGENCE', _data!.enrichedDescription),
            const SizedBox(height: 32),
            _listSection('LEARNED VOCABULARY', _data!.learnedPhrases),
            const SizedBox(height: 32),
            _listSection('OPERATIONAL RULES', _data!.additionalRules),
          ],
        ),
      ),
    );
  }

  Widget _section(String title, String content) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(title, style: const TextStyle(color: Color(AppConfig.emerald), fontSize: 9, fontWeight: FontWeight.w900, letterSpacing: 2)),
        const SizedBox(height: 16),
        Container(
          padding: const EdgeInsets.all(20),
          decoration: BoxDecoration(color: Colors.white.withValues(alpha: 0.02), borderRadius: BorderRadius.circular(16), border: Border.all(color: Colors.white10)),
          child: Text(content, style: const TextStyle(color: Colors.white70, fontSize: 13, height: 1.6)),
        ),
      ],
    );
  }

  Widget _listSection(String title, List<String> items) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(title, style: const TextStyle(color: Color(AppConfig.emerald), fontSize: 9, fontWeight: FontWeight.w900, letterSpacing: 2)),
        const SizedBox(height: 16),
        ...items.map((item) => Container(
          margin: const EdgeInsets.only(bottom: 8),
          padding: const EdgeInsets.all(12),
          decoration: BoxDecoration(color: Colors.white.withValues(alpha: 0.02), borderRadius: BorderRadius.circular(12)),
          child: Row(
            children: [
              const Icon(LucideIcons.checkCircle2, color: Color(AppConfig.emerald), size: 12),
              const SizedBox(width: 12),
              Expanded(child: Text(item, style: const TextStyle(color: Colors.white70, fontSize: 12))),
            ],
          ),
        )),
        if (items.isEmpty)
          const Text('No dynamic patterns learned yet.', style: TextStyle(color: Colors.white24, fontSize: 11, fontStyle: FontStyle.italic)),
      ],
    );
  }

  Widget _buildSimulationPane() {
    return Container(
      color: Colors.black.withValues(alpha: 0.2),
      padding: const EdgeInsets.all(32),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text('ALIGNMENT PLAYGROUND', style: TextStyle(color: Colors.blue, fontSize: 10, fontWeight: FontWeight.w900, letterSpacing: 2)),
          const SizedBox(height: 24),
          
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 8),
            decoration: BoxDecoration(
              color: Colors.white.withValues(alpha: 0.02),
              borderRadius: BorderRadius.circular(16),
              border: Border.all(color: Colors.white10),
            ),
            child: TextField(
              controller: _simController,
              onSubmitted: (_) => _runSimulation(),
              style: const TextStyle(color: Colors.white, fontSize: 14),
              decoration: InputDecoration(
                hintText: 'Simulate natural language interaction...',
                hintStyle: const TextStyle(color: Colors.white24),
                border: InputBorder.none,
                suffixIcon: IconButton(
                  onPressed: _isSimulating ? null : _runSimulation,
                  icon: _isSimulating 
                    ? const SizedBox(width: 16, height: 16, child: CircularProgressIndicator(strokeWidth: 2))
                    : const Icon(LucideIcons.play, color: Colors.blue, size: 16),
                ),
              ),
            ),
          ),
          
          if (_simOutcome != null) ...[
            const SizedBox(height: 32),
            const Text('SIMULATION TRACE', style: TextStyle(color: Colors.white24, fontSize: 8, fontWeight: FontWeight.w900, letterSpacing: 1)),
            const SizedBox(height: 16),
            Expanded(
              child: Container(
                width: double.infinity,
                padding: const EdgeInsets.all(24),
                decoration: BoxDecoration(
                  color: Colors.black,
                  borderRadius: BorderRadius.circular(20),
                  border: Border.all(color: Colors.white10),
                ),
                child: SingleChildScrollView(
                  child: Text(
                    _simOutcome!,
                    style: const TextStyle(color: Color(AppConfig.emerald), fontSize: 13, fontFamily: 'monospace', height: 1.5),
                  ),
                ),
              ),
            ),
          ],
        ],
      ),
    );
  }
}
