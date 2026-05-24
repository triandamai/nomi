import 'dart:io';
import 'dart:ui';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';
import 'package:image_picker/image_picker.dart';
import 'package:nomi_mobile/providers/chat_provider.dart';
import 'package:nomi_mobile/providers/repositories.dart';
import 'package:nomi_mobile/data/models/message.dart';
import 'package:nomi_mobile/ui/widgets/chat_bubble.dart';
import 'package:nomi_mobile/ui/widgets/sidebar.dart';
import 'package:nomi_mobile/core/config.dart';
import 'package:nomi_mobile/core/utils/formatter.dart';
import 'package:nomi_mobile/ui/widgets/typing_indicator.dart';
import 'package:nomi_mobile/core/db/database.dart' as db;

class ChatPage extends ConsumerStatefulWidget {
  const ChatPage({super.key});

  @override
  ConsumerState<ChatPage> createState() => _ChatPageState();
}

class _ChatPageState extends ConsumerState<ChatPage> {
  final _textController = TextEditingController();
  final _scrollController = ScrollController();
  final _imagePicker = ImagePicker();
  bool _showScrollToBottom = false;
  XFile? _selectedFile;
  bool _isUploading = false;

  @override
  void initState() {
    super.initState();
    _scrollController.addListener(_scrollListener);
    WidgetsBinding.instance.addPostFrameCallback((_) {
      ref.read(chatProvider.notifier).fetchConversations();
    });
  }

  @override
  void dispose() {
    _scrollController.removeListener(_scrollListener);
    _scrollController.dispose();
    _textController.dispose();
    super.dispose();
  }

  void _scrollListener() {
    if (_scrollController.hasClients) {
      final bool show = _scrollController.offset > 200;
      if (show != _showScrollToBottom) {
        setState(() => _showScrollToBottom = show);
      }
    }
  }

  void _scrollToBottom() {
    if (_scrollController.hasClients) {
      _scrollController.animateTo(
        0,
        duration: const Duration(milliseconds: 300),
        curve: Curves.easeOut,
      );
    }
  }

  Future<void> _pickMedia() async {
    final source = await showModalBottomSheet<ImageSource>(
      context: context,
      backgroundColor: const Color(AppConfig.deepSlate),
      shape: const RoundedRectangleBorder(borderRadius: BorderRadius.zero),
      builder: (context) => SafeArea(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            ListTile(
              leading: const Icon(LucideIcons.camera, color: Color(AppConfig.blue)),
              title: const Text('Take Photo', style: TextStyle(color: Colors.white)),
              onTap: () => Navigator.pop(context, ImageSource.camera),
            ),
            ListTile(
              leading: const Icon(LucideIcons.image, color: Color(AppConfig.emerald)),
              title: const Text('Choose from Gallery', style: TextStyle(color: Colors.white)),
              onTap: () => Navigator.pop(context, ImageSource.gallery),
            ),
          ],
        ),
      ),
    );

    if (source != null) {
      final file = await _imagePicker.pickImage(source: source, imageQuality: 70);
      if (file != null) {
        setState(() => _selectedFile = file);
      }
    }
  }

  Future<void> _handleSend() async {
    if (_textController.text.isEmpty && _selectedFile == null) return;

    final String content = _textController.text;
    Map<String, String>? media;

    if (_selectedFile != null) {
      setState(() => _isUploading = true);
      try {
        final res = await ref.read(fileRepositoryProvider).uploadFile(File(_selectedFile!.path));
        if (res.meta.isSuccess && res.data != null) {
          final String url = res.data!;
          if (_selectedFile!.name.toLowerCase().endsWith('.webp')) {
            media = {'sticker_url': url};
          } else {
            media = {'image_url': url};
          }
        }
      } catch (e) {
        print('Upload failed: $e');
      } finally {
        if (mounted) setState(() => _isUploading = false);
      }
    }

    ref.read(chatProvider.notifier).sendMessage(content, media: media);
    _textController.clear();
    setState(() => _selectedFile = null);
    _scrollToBottom();
  }

  @override
  Widget build(BuildContext context) {
    final chatState = ref.watch(chatProvider);
    final activeConvId = chatState.activeConversationId;
    final size = MediaQuery.of(context).size;
    final bool isLargeScreen = size.width >= 900;

    final messagesAsync = activeConvId != null 
        ? ref.watch(messagesStreamProvider(activeConvId))
        : const AsyncValue<List<db.Message>>.data([]);

    // Auto-scroll listener for real-time events
    ref.listen(chatProvider, (previous, next) {
      final cid = next.activeConversationId;
      if (cid == null) return;

      final wasTyping = previous?.isTyping[cid] ?? false;
      final isTyping = next.isTyping[cid] ?? false;
      
      final hadThought = previous?.thoughts[cid]?.isNotEmpty ?? false;
      final hasThought = next.thoughts[cid]?.isNotEmpty ?? false;

      final hadTool = previous?.activeTools[cid] != null;
      final hasTool = next.activeTools[cid] != null;

      if ((isTyping && !wasTyping) || 
          (hasThought && !hadThought) || 
          (hasTool && !hadTool)) {
        Future.delayed(const Duration(milliseconds: 100), () => _scrollToBottom());
      }
    });
    
    // Also scroll when new messages arrive from DB
    ref.listen(messagesStreamProvider(activeConvId ?? ""), (previous, next) {
      final prevCount = previous?.asData?.value.length ?? 0;
      final nextCount = next.asData?.value.length ?? 0;
      if (nextCount > prevCount) {
        Future.delayed(const Duration(milliseconds: 100), () => _scrollToBottom());
      }
    });

    final bool hasActiveThought = chatState.thoughts[activeConvId]?.isNotEmpty ?? false;
    final bool hasActiveTool = chatState.activeTools[activeConvId] != null;
    final bool showActiveProcess = hasActiveThought || hasActiveTool || (chatState.isTyping[activeConvId] ?? false);
    
    db.Conversation? activeConv;
    try {
      final conversations = ref.watch(conversationsStreamProvider).asData?.value ?? [];
      activeConv = conversations.firstWhere(
        (c) => c.id == activeConvId,
      );
    } catch (_) {}

    return Scaffold(
      backgroundColor: const Color(AppConfig.deepSlate),
      drawer: isLargeScreen ? null : const NomiSidebar(),
      floatingActionButton: _showScrollToBottom
          ? Padding(
              padding: const EdgeInsets.only(bottom: 45),
              child: FloatingActionButton.small(
                onPressed: _scrollToBottom,
                backgroundColor: const Color(AppConfig.blue).withValues(alpha: 0.8),
                elevation: 4,
                shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
                child: const Icon(LucideIcons.chevronDown, color: Colors.white, size: 20),
              ),
            )
          : null,
      floatingActionButtonLocation: FloatingActionButtonLocation.endFloat,
      appBar: isLargeScreen
          ? null
          : AppBar(
              backgroundColor: const Color(AppConfig.deepSlate).withValues(alpha: 0.8),
              elevation: 0,
              title: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      Expanded(
                        child: Text(
                          activeConv?.name ?? 'Nomi Chat',
                          style: const TextStyle(fontSize: 16, fontWeight: FontWeight.bold),
                          overflow: TextOverflow.ellipsis,
                        ),
                      ),
                      if (activeConv?.cumulativeTokens != null)
                        Padding(
                          padding: const EdgeInsets.only(left: 8),
                          child: Text(
                            '${Formatter.formatTokenCount(activeConv!.cumulativeTokens)} TOKENS',
                            style: const TextStyle(
                              color: Color(AppConfig.blue),
                              fontSize: 8,
                              fontWeight: FontWeight.w900,
                              letterSpacing: 1,
                              fontFamily: 'monospace',
                            ),
                          ),
                        ),
                    ],
                  ),
                  if (chatState.isTyping[chatState.activeConversationId] ?? false)
                    const Text(
                      'Nomi is typing...',
                      style: TextStyle(fontSize: 10, color: Color(AppConfig.blue), fontWeight: FontWeight.bold),
                    ),
                ],
              ),
              actions: [
                IconButton(
                  onPressed: () {
                    if (chatState.activeConversationId != null) {
                      ref.read(chatProvider.notifier).fetchMessages(chatState.activeConversationId!);
                    }
                  },
                  icon: const Icon(LucideIcons.refreshCw, size: 18),
                ),
              ],
            ),
      body: Stack(
        children: [
          Positioned.fill(
            child: Opacity(
              opacity: 0.05,
              child: Image.asset(
                'assets/images/bg_dark.png',
                repeat: ImageRepeat.repeat,
              ),
            ),
          ),
          
          // 💬 Chat Content Layer (Shifted for Permanent Rail)
          GestureDetector(
            behavior: HitTestBehavior.translucent,
            onTap: () {
              if (isLargeScreen && chatState.isSidebarExpanded) {
                ref.read(chatProvider.notifier).toggleSidebar();
              }
            },
            child: Padding(
              padding: EdgeInsets.only(left: isLargeScreen ? 72 : 0),
              child: Column(
              children: [
                // 🏗️ Full-Width Header
                if (isLargeScreen)
                  Container(
                    width: double.infinity,
                    padding: const EdgeInsets.fromLTRB(24, 24, 24, 12),
                    decoration: BoxDecoration(
                      color: const Color(AppConfig.deepSlate),
                      border: Border(bottom: BorderSide(color: Colors.white.withValues(alpha: 0.05))),
                    ),
                    child: Row(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      children: [
                        Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Row(
                              children: [
                                Text(
                                  activeConv?.name ?? 'Nomi Chat',
                                  style: const TextStyle(color: Colors.white, fontSize: 24, fontWeight: FontWeight.w900),
                                ),
                                if (activeConv?.cumulativeTokens != null) ...[
                                  const SizedBox(width: 12),
                                  Container(
                                    padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
                                    decoration: BoxDecoration(
                                      color: const Color(AppConfig.blue).withValues(alpha: 0.1),
                                      borderRadius: BorderRadius.circular(6),
                                      border: Border.all(color: const Color(AppConfig.blue).withValues(alpha: 0.2)),
                                    ),
                                    child: Text(
                                      '${Formatter.formatTokenCount(activeConv!.cumulativeTokens)} CUMULATIVE TOKENS',
                                      style: const TextStyle(
                                        color: Color(AppConfig.blue),
                                        fontSize: 9,
                                        fontWeight: FontWeight.w900,
                                        letterSpacing: 1.2,
                                        fontFamily: 'monospace',
                                      ),
                                    ),
                                  ),
                                ],
                              ],
                            ),
                            if (chatState.isTyping[activeConvId] ?? false)
                              const Text(
                                'Nomi is typing...',
                                style: TextStyle(fontSize: 11, color: Color(AppConfig.blue), fontWeight: FontWeight.w900),
                              ),
                          ],
                        ),
                        IconButton(
                          onPressed: () {
                            if (chatState.activeConversationId != null) {
                              ref.read(chatProvider.notifier).fetchMessages(chatState.activeConversationId!);
                            }
                          },
                          icon: const Icon(LucideIcons.refreshCw, size: 20, color: Colors.white24),
                        ),
                      ],
                    ),
                  ),

                // 💬 Centered Content Area
                Expanded(
                  child: activeConvId == null
                      ? _buildEmptyState(isLargeScreen)
                      : Center(
                          child: Container(
                            constraints: BoxConstraints(maxWidth: isLargeScreen ? 800 : double.infinity),
                            child: Stack(
                              children: [
                                Column(
                                  children: [
                                    Expanded(
                                      child: messagesAsync.when(
                                        data: (dbMessages) {
                                          final uiMessages = dbMessages.map((m) => Message.fromDb(m)).toList();
                                          return ListView.builder(
                                            controller: _scrollController,
                                            reverse: true,
                                            padding: EdgeInsets.fromLTRB(
                                              isLargeScreen ? 32 : 16,
                                              16,
                                              isLargeScreen ? 32 : 16,
                                              100,
                                            ),
                                            itemCount: uiMessages.length + (showActiveProcess ? 1 : 0),
                                            itemBuilder: (context, index) {
                                              if (showActiveProcess && index == 0) {
                                                return _buildActiveProcess(chatState, activeConvId);
                                              }

                                              final messageIndex = showActiveProcess ? index - 1 : index;
                                              final message = uiMessages[messageIndex];
                                              return ChatBubble(
                                                message: message,
                                                onReply: () => ref.read(chatProvider.notifier).setReplyingTo(message),
                                              );
                                            },
                                          );
                                        },
                                        loading: () => const Center(child: CircularProgressIndicator(strokeWidth: 2)),
                                        error: (e, _) => Center(child: Text('Sync Error: $e', style: const TextStyle(color: Colors.red))),
                                      ),
                                    ),
                                  ],
                                ),
                                
                                Positioned(
                                  bottom: 0,
                                  left: 0,
                                  right: 0,
                                  child: _buildInputArea(chatState, isLargeScreen),
                                ),
                              ],
                            ),
                          ),
                        ),
                ),
              ],
            ),
          ),
        ),

        // 🏗️ Overlaid Sidebar (Permanent Rail + Floating Drawer)
          if (isLargeScreen)
            const NomiSidebar(isDrawer: false),
        ],
      ),
    );
  }

  Widget _buildEmptyState(bool isLargeScreen) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Container(
            width: 80,
            height: 80,
            decoration: BoxDecoration(
              color: const Color(0xFF3b82f6),
              borderRadius: BorderRadius.circular(24),
              boxShadow: [
                BoxShadow(
                  color: const Color(0xFF3b82f6).withValues(alpha: 0.3),
                  blurRadius: 20,
                  offset: const Offset(0, 8),
                )
              ],
            ),
            child: const Center(
              child: Text(
                'N',
                style: TextStyle(color: Colors.white, fontSize: 40, fontWeight: FontWeight.w900),
              ),
            ),
          ),
          const SizedBox(height: 24),
          const Text(
            'Select a Session',
            style: TextStyle(color: Colors.white, fontSize: 24, fontWeight: FontWeight.w900, letterSpacing: -0.5),
          ),
          const SizedBox(height: 8),
          Text(
            'Choose a conversation from the sidebar to begin technical operations.',
            textAlign: TextAlign.center,
            style: TextStyle(color: Colors.white.withValues(alpha: 0.3), fontSize: 13, fontWeight: FontWeight.bold),
          ),
          if (!isLargeScreen) ...[
            const SizedBox(height: 32),
            ElevatedButton.icon(
              onPressed: () => Scaffold.of(context).openDrawer(),
              icon: const Icon(LucideIcons.menu, size: 16),
              label: const Text('OPEN NAVIGATOR', style: TextStyle(fontSize: 10, fontWeight: FontWeight.w900, letterSpacing: 1)),
              style: ElevatedButton.styleFrom(
                backgroundColor: Colors.blue.withValues(alpha: 0.1),
                foregroundColor: Colors.blue,
                elevation: 0,
                padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 16),
                shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
                side: BorderSide(color: Colors.blue.withValues(alpha: 0.2)),
              ),
            ),
          ],
        ],
      ),
    );
  }

  Widget _buildActiveProcess(ChatState chatState, String? convId) {
    final thought = chatState.thoughts[convId];
    final tool = chatState.activeTools[convId];
    final isTyping = chatState.isTyping[convId] ?? false;

    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 8),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          if (isTyping || (thought != null && thought.isNotEmpty) || tool != null)
            Padding(
              padding: const EdgeInsets.only(left: 4, bottom: 6),
              child: Row(
                children: [
                  const Text(
                    'Nomi',
                    style: TextStyle(
                      color: Color(AppConfig.blue),
                      fontSize: 12,
                      fontWeight: FontWeight.w900,
                      letterSpacing: 0.5,
                    ),
                  ),
                  if (tool != null) ...[
                    const SizedBox(width: 8),
                    _buildToolIndicator(tool),
                  ],
                ],
              ),
            ),

          if (isTyping)
            const Padding(
              padding: EdgeInsets.only(bottom: 8),
              child: TypingIndicator(color: Color(AppConfig.blue)),
            ),

          if (thought != null && thought.isNotEmpty)
            _buildStreamingThought(thought),
        ],
      ),
    );
  }

  Widget _buildStreamingThought(String thought) {
    return Container(
      margin: const EdgeInsets.only(bottom: 8),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: const Color(AppConfig.blue).withValues(alpha: 0.05),
        borderRadius: const BorderRadius.only(
          topLeft: Radius.circular(16),
          topRight: Radius.circular(16),
          bottomLeft: Radius.circular(4),
          bottomRight: Radius.circular(16),
        ),
        border: Border.all(color: const Color(AppConfig.blue).withValues(alpha: 0.1)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Row(
            children: [
              Icon(LucideIcons.brain, size: 12, color: Color(AppConfig.blue)),
              SizedBox(width: 8),
              Text(
                'THINKING...',
                style: TextStyle(
                  color: Color(AppConfig.blue),
                  fontSize: 8,
                  fontWeight: FontWeight.w900,
                  letterSpacing: 1.5,
                ),
              ),
            ],
          ),
          const SizedBox(height: 8),
          Text(
            thought,
            style: TextStyle(
              color: Colors.white.withValues(alpha: 0.7),
              fontSize: 13,
              fontStyle: FontStyle.italic,
              height: 1.4,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildToolIndicator(String toolName) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
      decoration: BoxDecoration(
        color: const Color(AppConfig.blue).withValues(alpha: 0.1),
        borderRadius: BorderRadius.circular(4),
        border: Border.all(color: const Color(AppConfig.blue).withValues(alpha: 0.2)),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Icon(LucideIcons.wrench, size: 10, color: Color(AppConfig.blue)),
          const SizedBox(width: 6),
          Text(
            'USING ${toolName.toUpperCase()}',
            style: const TextStyle(
              color: Color(AppConfig.blue),
              fontSize: 8,
              fontWeight: FontWeight.w900,
              letterSpacing: 0.5,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildInputArea(ChatState chatState, bool isLargeScreen) {
    return Padding(
      padding: EdgeInsets.fromLTRB(
        isLargeScreen ? 24 : 16,
        8,
        isLargeScreen ? 24 : 16,
        isLargeScreen ? 32 : 16,
      ),
      child: ClipRRect(
        borderRadius: BorderRadius.circular(24),
        child: BackdropFilter(
          filter: ImageFilter.blur(sigmaX: 12, sigmaY: 12),
          child: Container(
            padding: const EdgeInsets.all(8),
            decoration: BoxDecoration(
              color: Colors.transparent,
              borderRadius: BorderRadius.circular(24),
              border: Border.all(color: Colors.white.withValues(alpha: 0.1)),
            ),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                if (_selectedFile != null)
                  _buildFilePreview(),
                
                if (chatState.replyingTo != null)
                  _buildReplyContext(chatState.replyingTo!),

                Row(
                  children: [
                    IconButton(
                      onPressed: _isUploading ? null : _pickMedia,
                      icon: Icon(
                        _selectedFile != null ? LucideIcons.checkCircle2 : LucideIcons.paperclip,
                        color: _selectedFile != null ? const Color(AppConfig.emerald) : Colors.white38,
                      ),
                    ),
                    Expanded(
                      child: Container(
                        padding: const EdgeInsets.symmetric(horizontal: 16),
                        decoration: BoxDecoration(
                          color: Colors.white.withValues(alpha: 0.03),
                          borderRadius: BorderRadius.circular(20),
                        ),
                        child: TextField(
                          controller: _textController,
                          onSubmitted: (_) => _handleSend(),
                          style: const TextStyle(color: Colors.white, fontSize: 14),
                          decoration: const InputDecoration(
                            hintText: 'Message Nomi...',
                            hintStyle: TextStyle(color: Colors.white24, fontSize: 14),
                            border: InputBorder.none,
                          ),
                        ),
                      ),
                    ),
                    const SizedBox(width: 8),
                    CircleAvatar(
                      backgroundColor: _isUploading ? Colors.white10 : const Color(AppConfig.blue),
                      radius: 20,
                      child: IconButton(
                        onPressed: _isUploading ? null : _handleSend,
                        icon: _isUploading 
                          ? const SizedBox(width: 18, height: 18, child: CircularProgressIndicator(strokeWidth: 2, color: Color(AppConfig.blue)))
                          : const Icon(LucideIcons.send, color: Colors.white, size: 16),
                      ),
                    ),
                  ],
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildFilePreview() {
    return Container(
      margin: const EdgeInsets.only(bottom: 12),
      padding: const EdgeInsets.all(8),
      decoration: BoxDecoration(
        color: Colors.white.withValues(alpha: 0.05),
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: Colors.white10),
      ),
      child: Row(
        children: [
          ClipRRect(
            borderRadius: BorderRadius.circular(8),
            child: Image.file(File(_selectedFile!.path), width: 40, height: 40, fit: BoxFit.cover),
          ),
          const SizedBox(width: 12),
          Expanded(
            child: Text(
              _selectedFile!.name,
              style: const TextStyle(color: Colors.white70, fontSize: 12, fontWeight: FontWeight.bold),
              overflow: TextOverflow.ellipsis,
            ),
          ),
          IconButton(
            onPressed: () => setState(() => _selectedFile = null),
            icon: const Icon(LucideIcons.x, size: 16, color: Colors.white38),
          ),
        ],
      ),
    );
  }

  Widget _buildReplyContext(Message message) {
    return Container(
      margin: const EdgeInsets.only(bottom: 12),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: const Color(AppConfig.blue).withValues(alpha: 0.05),
        borderRadius: BorderRadius.circular(16),
        border: Border.all(color: const Color(AppConfig.blue).withValues(alpha: 0.2)),
      ),
      child: Row(
        children: [
          const Icon(LucideIcons.reply, size: 14, color: Color(AppConfig.blue)),
          const SizedBox(width: 12),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Replying to ${message.displayName ?? message.role}',
                  style: const TextStyle(color: Color(AppConfig.blue), fontSize: 9, fontWeight: FontWeight.bold),
                ),
                Text(
                  message.content,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: TextStyle(color: Colors.white.withValues(alpha: 0.5), fontSize: 11),
                ),
              ],
            ),
          ),
          IconButton(
            onPressed: () => ref.read(chatProvider.notifier).setReplyingTo(null),
            icon: const Icon(LucideIcons.x, size: 14, color: Colors.white38),
          ),
        ],
      ),
    );
  }
}
