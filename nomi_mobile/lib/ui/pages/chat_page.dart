import 'dart:io';
import 'dart:ui';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';
import 'package:image_picker/image_picker.dart';
import 'package:nomi_mobile/providers/chat_provider.dart';
import 'package:nomi_mobile/providers/auth_provider.dart';
import 'package:nomi_mobile/providers/repositories.dart';
import 'package:nomi_mobile/data/models/message.dart';
import 'package:nomi_mobile/ui/widgets/chat_bubble.dart';
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

  List<dynamic> _mentionCandidates = [];
  bool _showMentionSuggestions = false;
  int _mentionStartIdx = -1;
  final Map<String, String> _selectedMentions = {};
  List<dynamic> _conversationMembers = [];

  Future<void> _fetchConversationMembers(String conversationId) async {
    try {
      final apiClient = ref.read(apiClientProvider);
      final response = await apiClient.dio.get('/conversations/$conversationId/members');
      if (response.statusCode == 200 && response.data != null && response.data['data'] != null) {
        final List<dynamic> members = response.data['data'] ?? [];
        if (mounted) {
          setState(() {
            _conversationMembers = members;
          });
        }
      }
    } catch (e) {
      debugPrint('Failed to fetch conversation members: $e');
    }
  }

  void _onTextChanged(String text) {
    final selection = _textController.selection;
    if (!selection.isValid || !selection.isCollapsed) {
      setState(() => _showMentionSuggestions = false);
      return;
    }

    final cursorIdx = selection.baseOffset;
    final textBeforeCursor = text.substring(0, cursorIdx);
    final lastAtIdx = textBeforeCursor.lastIndexOf('@');

    if (lastAtIdx != -1) {
      final query = textBeforeCursor.substring(lastAtIdx + 1);
      // Ensure there are no spaces in the query and we are immediately after @
      if (!query.contains(' ')) {
        setState(() {
          _showMentionSuggestions = true;
          _mentionStartIdx = lastAtIdx;
        });
        _searchMentions(query);
        return;
      }
    }

    setState(() => _showMentionSuggestions = false);
  }

  Future<void> _searchMentions(String query) async {
    final currentUserId = ref.read(authProvider).user?.id;
    final term = query.toLowerCase();

    // 1. Get initial local matches from _conversationMembers
    final seenIds = <String>{};
    final localMatches = <dynamic>[];
    for (final member in _conversationMembers) {
      final userId = member['user_id']?.toString() ?? member['id']?.toString();
      if (userId != null && userId != currentUserId && !seenIds.contains(userId)) {
        final String displayName = (member['display_name']?.toString() ?? '').toLowerCase();
        final String username = (member['username']?.toString() ?? '').toLowerCase();
        final String extId = (member['external_id']?.toString() ?? '').toLowerCase();

        if (displayName.contains(term) || username.contains(term) || extId.contains(term)) {
          seenIds.add(userId);
          localMatches.add(member);
        }
      }
    }

    // Set local candidates immediately so the UI is highly reactive
    setState(() {
      _mentionCandidates = List.from(localMatches);
    });

    // 2. Fetch global matches in background to enrich results and support private chats
    try {
      final apiClient = ref.read(apiClientProvider);
      // Fix: Query parameter key must be 'q' to align with the Rust backend!
      final response = await apiClient.dio.get('/users/search', queryParameters: {'q': query});
      
      if (response.statusCode == 200 && response.data != null && response.data['data'] != null) {
        final List<dynamic> globalItems = response.data['data'] ?? [];
        
        // Merge global items that aren't duplicates and aren't self
        final merged = List.from(localMatches);
        for (final g in globalItems) {
          final userId = g['user_id']?.toString() ?? g['id']?.toString();
          if (userId != null && userId != currentUserId && !seenIds.contains(userId)) {
            seenIds.add(userId);
            merged.add(g);
          }
        }

        // Only update state if the user is still looking at suggestions for this specific query
        if (mounted && _showMentionSuggestions) {
          setState(() {
            _mentionCandidates = merged;
          });
        }
      }
    } catch (e) {
      debugPrint('Global search mentions failed: $e');
    }
  }

  void _selectMention(dynamic user) {
    final text = _textController.text;
    final String identifier = user['external_id']?.toString() ?? user['username']?.toString() ?? user['id']?.toString() ?? '';
    if (identifier.isEmpty) return;

    String displayName = user['display_name']?.toString() ?? '';
    // Strip JID/LID '@...' suffixes if they exist in the display name
    if (displayName.contains('@')) {
      displayName = displayName.split('@')[0];
    }
    // Fallback to cleaned external ID / username if display name is not populated
    if (displayName.isEmpty) {
      displayName = identifier.contains('@') ? identifier.split('@')[0] : identifier;
    }
    if (displayName.isEmpty) {
      displayName = 'User';
    }

    final cursorIdx = _textController.selection.baseOffset;
    final prefix = text.substring(0, _mentionStartIdx);
    final suffix = text.substring(cursorIdx);

    // Friendly display: e.g. "@Trian" or "@628123456789"
    final String friendlyMention = '@$displayName';
    
    // Save to our state map for raw ID translation upon sending!
    _selectedMentions[friendlyMention] = identifier;

    final replacement = '$friendlyMention ';
    _textController.text = '$prefix$replacement$suffix';

    // Move cursor to the end of the replaced mention
    _textController.selection = TextSelection.collapsed(
      offset: _mentionStartIdx + replacement.length,
    );

    setState(() {
      _showMentionSuggestions = false;
      _mentionCandidates = [];
    });
  }

  @override
  void initState() {
    super.initState();
    _scrollController.addListener(_scrollListener);
    WidgetsBinding.instance.addPostFrameCallback((_) {
      ref.read(chatProvider.notifier).fetchConversations();
      final activeConvId = ref.read(chatProvider).activeConversationId;
      if (activeConvId != null) {
        _fetchConversationMembers(activeConvId);
      }
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

    String content = _textController.text;
    
    // Convert friendly mentions (e.g., "@TRIAN") to raw identifiers (e.g., "@166168073642181")
    _selectedMentions.forEach((friendly, rawId) {
      final escaped = RegExp.escape(friendly);
      final regex = RegExp('$escaped\\b');
      content = content.replaceAll(regex, '@$rawId');
    });

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
        debugPrint('Upload failed: $e');
      } finally {
        if (mounted) setState(() => _isUploading = false);
      }
    }

    ref.read(chatProvider.notifier).sendMessage(content, media: media);
    _textController.clear();
    _selectedMentions.clear(); // Clear the local state mapping
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

    ref.listen(chatProvider, (previous, next) {
      final cid = next.activeConversationId;
      if (cid == null) return;

      // Switch conversation members dynamically
      if (previous?.activeConversationId != cid) {
        _fetchConversationMembers(cid);
      }

      final wasTyping = previous?.isTyping[cid] ?? false;
      final isTyping = next.isTyping[cid] ?? false;
      final hadThought = previous?.thoughts[cid]?.isNotEmpty ?? false;
      final hasThought = next.thoughts[cid]?.isNotEmpty ?? false;
      final hadTool = previous?.activeTools[cid] != null;
      final hasTool = next.activeTools[cid] != null;

      if ((isTyping && !wasTyping) || (hasThought && !hadThought) || (hasTool && !hadTool)) {
        Future.delayed(const Duration(milliseconds: 100), () => _scrollToBottom());
      }
    });
    
    ref.listen(messagesStreamProvider(activeConvId ?? ""), (previous, next) {
      final prevCount = previous?.asData?.value.length ?? 0;
      final nextCount = next.asData?.value.length ?? 0;
      if (nextCount > prevCount) {
        Future.delayed(const Duration(milliseconds: 100), () => _scrollToBottom());
      }
    });

    final bool showActiveProcess = (chatState.thoughts[activeConvId]?.isNotEmpty ?? false) || 
                                  (chatState.activeTools[activeConvId] != null) || 
                                  (chatState.isTyping[activeConvId] ?? false);
    
    db.Conversation? activeConv;
    try {
      final conversations = ref.watch(conversationsStreamProvider).asData?.value ?? [];
      activeConv = conversations.firstWhere((c) => c.id == activeConvId);
    } catch (_) {}

    return Scaffold(
      backgroundColor: Colors.transparent,
      floatingActionButton: _showScrollToBottom
          ? Padding(
              padding: const EdgeInsets.only(bottom: 100),
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
              leading: IconButton(
                onPressed: () => Scaffold.of(context).openDrawer(),
                icon: const Icon(LucideIcons.menu),
              ),
              title: InkWell(
                onTap: activeConv != null ? () => _showThreadDetail(context, activeConv!) : null,
                borderRadius: BorderRadius.circular(8),
                child: Padding(
                  padding: const EdgeInsets.symmetric(vertical: 4, horizontal: 4),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Row(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          Flexible(
                            child: Text(
                              activeConv?.name ?? 'Nomi Chat',
                              style: const TextStyle(fontSize: 16, fontWeight: FontWeight.bold),
                              overflow: TextOverflow.ellipsis,
                            ),
                          ),
                          const SizedBox(width: 6),
                          const Icon(LucideIcons.info, size: 12, color: Colors.white24),
                        ],
                      ),
                      if (chatState.isTyping[chatState.activeConversationId] ?? false)
                        const Text(
                          'Nomi is typing...',
                          style: TextStyle(fontSize: 10, color: Color(AppConfig.blue), fontWeight: FontWeight.bold),
                        ),
                    ],
                  ),
                ),
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
              child: Image.asset('assets/images/bg_dark.png', repeat: ImageRepeat.repeat),
            ),
          ),
          activeConvId == null
              ? _buildEmptyState(isLargeScreen)
              : Column(
                  children: [
                    if (isLargeScreen) _buildDesktopHeader(activeConv, chatState, activeConvId),
                    Expanded(
                      child: Center(
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
                                          padding: EdgeInsets.fromLTRB(isLargeScreen ? 32 : 16, 16, isLargeScreen ? 32 : 16, 100),
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
        ],
      ),
    );
  }

  Widget _buildDesktopHeader(db.Conversation? activeConv, ChatState chatState, String? activeConvId) {
    return Container(
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
                  InkWell(
                    onTap: activeConv != null ? () => _showThreadDetail(context, activeConv) : null,
                    borderRadius: BorderRadius.circular(8),
                    child: Padding(
                      padding: const EdgeInsets.all(4.0),
                      child: Row(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          Text(activeConv?.name ?? 'Nomi Chat', style: const TextStyle(color: Colors.white, fontSize: 24, fontWeight: FontWeight.w900)),
                          const SizedBox(width: 8),
                          const Icon(LucideIcons.info, size: 16, color: Colors.white24),
                        ],
                      ),
                    ),
                  ),
                  if (activeConv?.cumulativeTokens != null)
                    Padding(
                      padding: const EdgeInsets.only(left: 12),
                      child: Container(
                        padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
                        decoration: BoxDecoration(
                          color: const Color(AppConfig.blue).withValues(alpha: 0.1),
                          borderRadius: BorderRadius.circular(6),
                          border: Border.all(color: const Color(AppConfig.blue).withValues(alpha: 0.2)),
                        ),
                        child: Text(
                          '${Formatter.formatTokenCount(activeConv!.cumulativeTokens)} CUMULATIVE TOKENS',
                          style: const TextStyle(color: Color(AppConfig.blue), fontSize: 9, fontWeight: FontWeight.w900, letterSpacing: 1.2, fontFamily: 'monospace'),
                        ),
                      ),
                    ),
                ],
              ),
              if (chatState.isTyping[activeConvId] ?? false)
                const Text('Nomi is typing...', style: TextStyle(fontSize: 11, color: Color(AppConfig.blue), fontWeight: FontWeight.w900)),
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
    );
  }

  Widget _buildEmptyState(bool isLargeScreen) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Container(
            width: 80, height: 80,
            decoration: BoxDecoration(
              color: const Color(0xFF3b82f6),
              borderRadius: BorderRadius.circular(24),
              boxShadow: [BoxShadow(color: const Color(0xFF3b82f6).withValues(alpha: 0.3), blurRadius: 20, offset: const Offset(0, 8))],
            ),
            child: const Center(child: Text('N', style: TextStyle(color: Colors.white, fontSize: 40, fontWeight: FontWeight.w900))),
          ),
          const SizedBox(height: 24),
          const Text('Select a Session', style: TextStyle(color: Colors.white, fontSize: 24, fontWeight: FontWeight.w900, letterSpacing: -0.5)),
          const SizedBox(height: 8),
          Text('Choose a conversation from the sidebar to begin technical operations.', textAlign: TextAlign.center, style: TextStyle(color: Colors.white.withValues(alpha: 0.3), fontSize: 13, fontWeight: FontWeight.bold)),
          if (!isLargeScreen) ...[
            const SizedBox(height: 32),
            ElevatedButton.icon(
              onPressed: () => Scaffold.of(context).openDrawer(),
              icon: const Icon(LucideIcons.menu, size: 16),
              label: const Text('OPEN NAVIGATOR', style: TextStyle(fontSize: 10, fontWeight: FontWeight.w900, letterSpacing: 1)),
              style: ElevatedButton.styleFrom(backgroundColor: Colors.blue.withValues(alpha: 0.1), foregroundColor: Colors.blue, elevation: 0, padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 16), shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)), side: BorderSide(color: Colors.blue.withValues(alpha: 0.2))),
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
      child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        if (isTyping || (thought != null && thought.isNotEmpty) || tool != null)
          Padding(padding: const EdgeInsets.only(left: 4, bottom: 6), child: Row(children: [const Text('Nomi', style: TextStyle(color: Color(AppConfig.blue), fontSize: 12, fontWeight: FontWeight.w900, letterSpacing: 0.5)), if (tool != null) ...[const SizedBox(width: 8), _buildToolIndicator(tool)]])),
        if (isTyping) const Padding(padding: EdgeInsets.only(bottom: 8), child: TypingIndicator(color: Color(AppConfig.blue))),
        if (thought != null && thought.isNotEmpty) _buildStreamingThought(thought),
      ]),
    );
  }

  Widget _buildStreamingThought(String thought) {
    return Container(
      margin: const EdgeInsets.only(bottom: 8),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(color: const Color(AppConfig.blue).withValues(alpha: 0.05), borderRadius: const BorderRadius.only(topLeft: Radius.circular(16), topRight: Radius.circular(16), bottomLeft: Radius.circular(4), bottomRight: Radius.circular(16)), border: Border.all(color: const Color(AppConfig.blue).withValues(alpha: 0.1))),
      child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        const Row(children: [Icon(LucideIcons.brain, size: 12, color: Color(AppConfig.blue)), SizedBox(width: 8), Text('THINKING...', style: TextStyle(color: Color(AppConfig.blue), fontSize: 8, fontWeight: FontWeight.w900, letterSpacing: 1.5))]),
        const SizedBox(height: 8),
        Text(thought, style: TextStyle(color: Colors.white.withValues(alpha: 0.7), fontSize: 13, fontStyle: FontStyle.italic, height: 1.4)),
      ]),
    );
  }

  Widget _buildToolIndicator(String toolName) {
    return Container(padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2), decoration: BoxDecoration(color: const Color(AppConfig.blue).withValues(alpha: 0.1), borderRadius: BorderRadius.circular(4), border: Border.all(color: const Color(AppConfig.blue).withValues(alpha: 0.2))), child: Row(mainAxisSize: MainAxisSize.min, children: [const Icon(LucideIcons.wrench, size: 10, color: Color(AppConfig.blue)), const SizedBox(width: 6), Text('USING ${toolName.toUpperCase()}', style: const TextStyle(color: Color(AppConfig.blue), fontSize: 8, fontWeight: FontWeight.w900, letterSpacing: 0.5))]));
  }

  Widget _buildInputArea(ChatState chatState, bool isLargeScreen) {
    return Padding(
      padding: EdgeInsets.fromLTRB(isLargeScreen ? 24 : 16, 8, isLargeScreen ? 24 : 16, isLargeScreen ? 32 : 16),
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
                if (_showMentionSuggestions && _mentionCandidates.isNotEmpty)
                  _buildMentionSuggestions(),
                if (_selectedFile != null) _buildFilePreview(),
                if (chatState.replyingTo != null) _buildReplyContext(chatState.replyingTo!),
                Row(
                  children: [
                    IconButton(
                      onPressed: _isUploading ? null : _pickMedia,
                      icon: Icon(_selectedFile != null ? LucideIcons.checkCircle2 : LucideIcons.paperclip, color: _selectedFile != null ? const Color(AppConfig.emerald) : Colors.white38),
                    ),
                    Expanded(
                      child: Container(
                        padding: const EdgeInsets.symmetric(horizontal: 16),
                        decoration: BoxDecoration(color: Colors.white.withValues(alpha: 0.03), borderRadius: BorderRadius.circular(20)),
                        child: TextField(
                          controller: _textController, 
                          onChanged: _onTextChanged,
                          onSubmitted: (_) => _handleSend(), 
                          style: const TextStyle(color: Colors.white, fontSize: 14), 
                          decoration: const InputDecoration(
                            hintText: 'Message Nomi...', 
                            hintStyle: TextStyle(color: Colors.white24, fontSize: 14), 
                            border: InputBorder.none
                          )
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

  Widget _buildMentionSuggestions() {
    return Container(
      constraints: const BoxConstraints(maxHeight: 180),
      margin: const EdgeInsets.only(bottom: 8),
      decoration: BoxDecoration(
        color: Colors.black.withValues(alpha: 0.3),
        borderRadius: BorderRadius.circular(16),
        border: Border.all(color: Colors.white.withValues(alpha: 0.05)),
      ),
      child: ListView.separated(
        shrinkWrap: true,
        padding: const EdgeInsets.symmetric(vertical: 8),
        itemCount: _mentionCandidates.length,
        separatorBuilder: (context, index) => const Divider(color: Colors.white10, height: 1),
        itemBuilder: (context, index) {
          final user = _mentionCandidates[index];
          final String username = user['username']?.toString() ?? '';
          final String externalId = user['external_id']?.toString() ?? '';
          
          String displayName = user['display_name']?.toString() ?? '';
          // Strip JID/LID '@...' suffixes if they exist in the display name
          if (displayName.contains('@')) {
            displayName = displayName.split('@')[0];
          }
          // Fallback to cleaned external ID / username if display name is not populated
          if (displayName.isEmpty) {
            final String rawId = externalId.isNotEmpty ? externalId : (username.isNotEmpty ? username : (user['id']?.toString() ?? ''));
            displayName = rawId.contains('@') ? rawId.split('@')[0] : rawId;
          }
          if (displayName.isEmpty) {
            displayName = 'User';
          }
          
          return InkWell(
            onTap: () => _selectMention(user),
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
              child: Row(
                children: [
                  CircleAvatar(
                    backgroundColor: const Color(AppConfig.blue).withValues(alpha: 0.1),
                    radius: 14,
                    child: Text(
                      displayName.isNotEmpty ? displayName[0].toUpperCase() : 'U',
                      style: const TextStyle(color: Color(AppConfig.blue), fontSize: 10, fontWeight: FontWeight.bold),
                    ),
                  ),
                  const SizedBox(width: 12),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          displayName,
                          style: const TextStyle(color: Colors.white, fontSize: 13, fontWeight: FontWeight.bold),
                        ),
                        if (username.isNotEmpty)
                          Text(
                            '@$username${externalId.isNotEmpty ? " • $externalId" : ""}',
                            style: const TextStyle(color: Colors.white38, fontSize: 10),
                          ),
                      ],
                    ),
                  ),
                ],
              ),
            ),
          );
        },
      ),
    );
  }

  Widget _buildFilePreview() {
    return Container(margin: const EdgeInsets.only(bottom: 12), padding: const EdgeInsets.all(8), decoration: BoxDecoration(color: Colors.white.withValues(alpha: 0.05), borderRadius: BorderRadius.circular(12), border: Border.all(color: Colors.white10)), child: Row(children: [ClipRRect(borderRadius: BorderRadius.circular(8), child: Image.file(File(_selectedFile!.path), width: 40, height: 40, fit: BoxFit.cover)), const SizedBox(width: 12), Expanded(child: Text(_selectedFile!.name, style: const TextStyle(color: Colors.white70, fontSize: 12, fontWeight: FontWeight.bold), overflow: TextOverflow.ellipsis)), IconButton(onPressed: () => setState(() => _selectedFile = null), icon: const Icon(LucideIcons.x, size: 16, color: Colors.white38))]));
  }

  Widget _buildReplyContext(Message message) {
    return Container(margin: const EdgeInsets.only(bottom: 12), padding: const EdgeInsets.all(12), decoration: BoxDecoration(color: const Color(AppConfig.blue).withValues(alpha: 0.05), borderRadius: BorderRadius.circular(16), border: Border.all(color: const Color(AppConfig.blue).withValues(alpha: 0.2))), child: Row(children: [const Icon(LucideIcons.reply, size: 14, color: Color(AppConfig.blue)), const SizedBox(width: 12), Expanded(child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [Text('Replying to ${message.displayName ?? message.role}', style: const TextStyle(color: Color(AppConfig.blue), fontSize: 9, fontWeight: FontWeight.bold)), Text(message.content, maxLines: 1, overflow: TextOverflow.ellipsis, style: TextStyle(color: Colors.white.withValues(alpha: 0.5), fontSize: 11))])), IconButton(onPressed: () => ref.read(chatProvider.notifier).setReplyingTo(null), icon: const Icon(LucideIcons.x, size: 14, color: Colors.white38))]));
  }

  void _showThreadDetail(BuildContext context, db.Conversation conv) {
    showModalBottomSheet(
      context: context,
      isScrollControlled: true,
      backgroundColor: Colors.transparent,
      builder: (context) => _ThreadDetailSheet(conv: conv),
    );
  }
}

class _ThreadDetailSheet extends StatelessWidget {
  final db.Conversation conv;
  const _ThreadDetailSheet({required this.conv});

  (String, Color, String) _getInteractionMode(double val) {
    if (val <= 0.25) return ('Proactive', const Color(AppConfig.emerald), '🏁');
    if (val <= 0.50) return ('Balanced', const Color(AppConfig.blue), '🤝');
    if (val <= 0.75) return ('Conservative', const Color(AppConfig.amber), '🛡️');
    return ('Silent Monitor', Colors.white38, '🤫');
  }

  (String, Color, String) _getIntentMode(double val) {
    if (val <= 0.40) return ('Experimental', const Color(AppConfig.indigo), '🧪');
    if (val <= 0.70) return ('Adaptive', const Color(AppConfig.blue), '🏎️');
    return ('Strict', const Color(AppConfig.rose), '📐');
  }

  (String, Color, String) _getGuardrailMode(double val) {
    if (val <= 0.50) return ('Permissive', const Color(AppConfig.emerald), '🔓');
    if (val <= 0.80) return ('Standard', const Color(AppConfig.blue), '👤');
    return ('Hardened Shield', const Color(AppConfig.rose), '🌋');
  }

  @override
  Widget build(BuildContext context) {
    // Note: In a real scenario, we'd add this column to Drift.
    // I'll assume standard defaults or empty if not present.
    final double interactionGate = 0.60; 
    final double intentClassification = 0.40;
    final double guardrails = 0.65;

    return ClipRRect(
      child: BackdropFilter(
        filter: ImageFilter.blur(sigmaX: 40, sigmaY: 40),
        child: Container(
          padding: const EdgeInsets.all(32),
          decoration: BoxDecoration(
            color: const Color(AppConfig.deepSlate).withValues(alpha: 0.9),
            borderRadius: const BorderRadius.vertical(top: Radius.circular(24)),
            border: const Border(top: BorderSide(color: Colors.white10)),
          ),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  const Icon(LucideIcons.terminal, color: Color(AppConfig.blue), size: 24),
                  const SizedBox(width: 16),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        const Text('THREAD CONFIGURATION', style: TextStyle(color: Color(AppConfig.blue), fontSize: 10, fontWeight: FontWeight.w900, letterSpacing: 2)),
                        Text(conv.name ?? 'Untitled Session', style: const TextStyle(color: Colors.white, fontSize: 20, fontWeight: FontWeight.bold)),
                      ],
                    ),
                  ),
                  IconButton(onPressed: () => Navigator.pop(context), icon: const Icon(LucideIcons.x, color: Colors.white38)),
                ],
              ),
              const SizedBox(height: 32),
              
              // Token Metrics
              Row(
                children: [
                  Expanded(
                    child: _buildMetricCard('TOTAL USAGE', Formatter.formatTokenCount(conv.cumulativeTokens ?? 0), const Color(AppConfig.blue)),
                  ),
                  const SizedBox(width: 16),
                  Expanded(
                    child: _buildMetricCard('LIMIT', Formatter.formatTokenCount(conv.maxTokenUsage ?? 0), Colors.white24),
                  ),
                ],
              ),

              const SizedBox(height: 32),
              const Divider(color: Colors.white10),
              const SizedBox(height: 24),
              const Text('BEHAVIOR BOUNDARIES (DEB)', style: TextStyle(color: Colors.white38, fontSize: 9, fontWeight: FontWeight.w900, letterSpacing: 1.5)),
              const SizedBox(height: 24),

              _buildReadOnlyBoundary('Sociability', interactionGate, _getInteractionMode(interactionGate)),
              const SizedBox(height: 20),
              _buildReadOnlyBoundary('Confidence', intentClassification, _getIntentMode(intentClassification)),
              const SizedBox(height: 20),
              _buildReadOnlyBoundary('Vigilance', guardrails, _getGuardrailMode(guardrails)),

              const SizedBox(height: 40),
              Center(
                child: Text(
                  'Parameters are optimized for this conversation context.',
                  style: TextStyle(color: Colors.white.withValues(alpha: 0.2), fontSize: 11, fontStyle: FontStyle.italic),
                ),
              ),
              const SizedBox(height: 12),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildMetricCard(String label, String value, Color color) {
    return Container(
      padding: const EdgeInsets.all(20),
      decoration: BoxDecoration(
        color: Colors.black.withValues(alpha: 0.2),
        borderRadius: BorderRadius.circular(16),
        border: Border.all(color: Colors.white.withValues(alpha: 0.05)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(label, style: const TextStyle(color: Colors.white38, fontSize: 8, fontWeight: FontWeight.w900, letterSpacing: 1)),
          const SizedBox(height: 8),
          Text(value, style: TextStyle(color: color, fontSize: 18, fontWeight: FontWeight.bold, fontFamily: 'monospace')),
        ],
      ),
    );
  }

  Widget _buildReadOnlyBoundary(String label, double value, (String, Color, String) mode) {
    return Column(
      children: [
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            Text(label, style: const TextStyle(color: Colors.white70, fontSize: 11, fontWeight: FontWeight.bold)),
            Text('${mode.$3} ${mode.$1} (${value.toStringAsFixed(2)})', style: TextStyle(color: mode.$2, fontSize: 10, fontWeight: FontWeight.w900, fontFamily: 'monospace')),
          ],
        ),
        const SizedBox(height: 10),
        ClipRRect(
          borderRadius: BorderRadius.circular(2),
          child: LinearProgressIndicator(
            value: value,
            backgroundColor: Colors.white.withValues(alpha: 0.05),
            valueColor: AlwaysStoppedAnimation<Color>(mode.$2.withValues(alpha: 0.5)),
            minHeight: 2,
          ),
        ),
      ],
    );
  }
}
