import 'dart:ui';
import 'package:flutter/material.dart';
import 'package:flutter_markdown/flutter_markdown.dart';
import 'package:nomi_mobile/data/models/message.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';
import 'package:nomi_mobile/ui/widgets/reminder_card.dart';
import 'package:nomi_mobile/ui/widgets/finance_card.dart';
import 'package:nomi_mobile/ui/widgets/proposal_card.dart';
import 'package:cached_network_image/cached_network_image.dart';
import 'package:nomi_mobile/core/config.dart';
import 'package:nomi_mobile/core/utils/formatter.dart';
import 'package:url_launcher/url_launcher.dart';
import 'package:audioplayers/audioplayers.dart';
import 'package:markdown/markdown.dart' as md;
import 'package:nomi_mobile/ui/widgets/code_block.dart';

class ChatBubble extends StatefulWidget {
  final Message message;
  final VoidCallback? onReply;

  const ChatBubble({
    super.key,
    required this.message,
    this.onReply,
  });

  @override
  State<ChatBubble> createState() => _ChatBubbleState();
}

class _ChatBubbleState extends State<ChatBubble> {
  bool _thoughtExpanded = false;

  String _getFileUrl(String? path) {
    if (path == null) return '';
    if (path.startsWith('http')) return path;
    return '${AppConfig.fileUrl}/$path';
  }

  @override
  Widget build(BuildContext context) {
    final message = widget.message;

    return Dismissible(
      key: Key('reply_${message.id}'),
      direction: DismissDirection.startToEnd,
      confirmDismiss: (direction) async {
        if (widget.onReply != null) widget.onReply!();
        return false;
      },
      background: Container(
        alignment: Alignment.centerLeft,
        padding: const EdgeInsets.only(left: 20),
        child: const Icon(LucideIcons.reply, color: Colors.blue, size: 20),
      ),
      child: Padding(
        padding: const EdgeInsets.symmetric(vertical: 12, horizontal: 16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start, // 🎯 ALL messages on the left
          children: [
            // 👤 Display Name (Top & Outside Bubble)
            Padding(
              padding: const EdgeInsets.only(left: 2, bottom: 4),
              child: Row(
                children: [
                  Text(
                    message.displayName ?? (message.isUser ? 'Human' : 'Nomi'),
                    style: TextStyle(
                      color: message.isUser ? const Color(0xFF94a3b8) : const Color(0xFF3b82f6),
                      fontSize: 12,
                      fontWeight: FontWeight.w900,
                      letterSpacing: 0.5,
                    ),
                  ),
                  if (!message.isUser && message.totalTokens != null && message.totalTokens! > 0) ...[
                    const SizedBox(width: 8),
                    Container(
                      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 1),
                      decoration: BoxDecoration(
                        color: Colors.white.withValues(alpha: 0.05),
                        borderRadius: BorderRadius.circular(4),
                        border: Border.all(color: Colors.white.withValues(alpha: 0.1)),
                      ),
                      child: Text(
                        '${Formatter.formatTokenCount(message.totalTokens)} TOKENS',
                        style: TextStyle(
                          color: Colors.white.withValues(alpha: 0.3),
                          fontSize: 7,
                          fontWeight: FontWeight.w900,
                          letterSpacing: 1,
                        ),
                      ),
                    ),
                  ],
                  const SizedBox(width: 8),
                  Text(
                    'Today at ${DateTime.now().hour}:${DateTime.now().minute.toString().padLeft(2, '0')}', 
                    style: TextStyle(
                      color: Colors.white.withValues(alpha: 0.1),
                      fontSize: 9,
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                ],
              ),
            ),

            // 🗨️ Quoted Message
            if (message.repliedMessage != null)
              _buildQuotedMessage(),

            // 📦 Main Content Container (Discord Style - No Background)
            Container(
              constraints: BoxConstraints(maxWidth: MediaQuery.of(context).size.width * 0.9),
              padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 2), // Reduced padding for transparent style
              decoration: const BoxDecoration(
                color: Colors.transparent,
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  // Deep Thoughts (Expandable)
                  if (message.thought != null && message.thought!.isNotEmpty)
                    _buildThoughts(),

                  // 🖼️ Media Rendering
                  if (message.imageUrl != null) _buildImage(context, _getFileUrl(message.imageUrl)),
                  if (message.sticker_url != null) _buildSticker(_getFileUrl(message.sticker_url)),
                  if (message.audio_url != null) _buildAudioPlayer(_getFileUrl(message.audio_url)),
                  if (message.video_url != null) _buildVideoLink(_getFileUrl(message.video_url)),
                  if (message.document_url != null) _buildDocumentLink(_getFileUrl(message.document_url)),

                  if (message.content.isNotEmpty)
                    MarkdownBody(
                      data: message.content,
                      selectable: true,
                      onTapLink: (text, href, title) {
                        if (href != null) launchUrl(Uri.parse(href));
                      },
                      builders: {
                        'pre': CodeElementBuilder(),
                      },
                      styleSheet: MarkdownStyleSheet(
                        p: const TextStyle(color: Color(0xFFdcddde), fontSize: 14, height: 1.4),
                        code: const TextStyle(
                          backgroundColor: Color(0xFF202225),
                          fontFamily: 'monospace',
                          fontSize: 12,
                          color: Color(0xFF10b981),
                        ),
                      ),
                    ),

                  if (message.metadata != null && message.metadata!['tool_ref_ids'] != null)
                    ..._buildArtifacts(message.metadata!['tool_ref_ids']),
                ],
              ),
            ),
            
            // ⬅️ Reply Button (Below Bubble, Left aligned)
            if (widget.onReply != null)
              Padding(
                padding: const EdgeInsets.only(top: 6),
                child: InkWell(
                  onTap: widget.onReply,
                  borderRadius: BorderRadius.circular(4),
                  child: Container(
                    padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                    decoration: BoxDecoration(
                      color: Colors.white.withValues(alpha: 0.03),
                      borderRadius: BorderRadius.circular(4),
                    ),
                    child: Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Icon(LucideIcons.reply, size: 10, color: Colors.white.withValues(alpha: 0.2)),
                        const SizedBox(width: 4),
                        Text(
                          'REPLY',
                          style: TextStyle(
                            color: Colors.white.withValues(alpha: 0.2),
                            fontSize: 8,
                            fontWeight: FontWeight.w900,
                            letterSpacing: 1,
                          ),
                        ),
                      ],
                    ),
                  ),
                ),
              ),
          ],
        ),
      ),
    );
  }

  Widget _buildImage(BuildContext context, String url) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: ClipRRect(
        borderRadius: BorderRadius.circular(8),
        child: CachedNetworkImage(
          imageUrl: url,
          placeholder: (context, url) => Container(
            height: 200,
            color: Colors.white10,
            child: const Center(child: CircularProgressIndicator(strokeWidth: 2)),
          ),
          errorWidget: (context, url, error) => const Icon(LucideIcons.imageOff, color: Colors.white24),
          fit: BoxFit.cover,
        ),
      ),
    );
  }

  Widget _buildSticker(String url) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: CachedNetworkImage(
        imageUrl: url,
        width: 120,
        height: 120,
        placeholder: (context, url) => const SizedBox(width: 120, height: 120),
        errorWidget: (context, url, error) => const Icon(LucideIcons.smile, color: Colors.white24),
      ),
    );
  }

  Widget _buildAudioPlayer(String url) {
    return _AudioPlayerWidget(url: url);
  }

  Widget _buildVideoLink(String url) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: InkWell(
        onTap: () => launchUrl(Uri.parse(url)),
        child: Container(
          padding: const EdgeInsets.all(12),
          decoration: BoxDecoration(
            color: const Color(0xFF202225),
            borderRadius: BorderRadius.circular(8),
            border: Border.all(color: Colors.white10),
          ),
          child: const Row(
            children: [
              Icon(LucideIcons.playCircle, color: Colors.blue),
              SizedBox(width: 12),
              Text('Play Video', style: TextStyle(color: Colors.white, fontSize: 13, fontWeight: FontWeight.bold)),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildDocumentLink(String url) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: InkWell(
        onTap: () => launchUrl(Uri.parse(url)),
        child: Container(
          padding: const EdgeInsets.all(12),
          decoration: BoxDecoration(
            color: Colors.white.withValues(alpha: 0.05),
            borderRadius: BorderRadius.circular(8),
            border: Border.all(color: Colors.white10),
          ),
          child: const Row(
            children: [
              Icon(LucideIcons.fileText, color: Colors.amber),
              SizedBox(width: 12),
              Expanded(
                child: Text(
                  'Open Document',
                  style: TextStyle(color: Colors.white, fontSize: 13, fontWeight: FontWeight.bold),
                  overflow: TextOverflow.ellipsis,
                ),
              ),
              Icon(LucideIcons.externalLink, size: 14, color: Colors.white24),
            ],
          ),
        ),
      ),
    );
  }

  List<Widget> _buildArtifacts(List<dynamic> toolRefs) {
    return toolRefs.map((ref) {
      final tool = ref['tool']?.toString().toLowerCase() ?? '';
      final refId = ref['ref_id']?.toString() ?? '';

      if (tool.contains('reminder') || tool.contains('schedule_task')) {
        return ReminderCard(refId: refId);
      } else if (tool.contains('finance') || tool.contains('expense')) {
        return FinanceCard(refId: refId);
      } else if (tool.contains('skill') || tool.contains('proposal')) {
        return PluginProposalCard(refId: refId);
      }
      return const SizedBox.shrink();
    }).toList();
  }

  Widget _buildQuotedMessage() {
    final q = widget.message.repliedMessage!;
    return Container(
      margin: const EdgeInsets.only(bottom: 8, left: 4),
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      decoration: BoxDecoration(
        color: Colors.white.withValues(alpha: 0.03),
        border: const Border(left: BorderSide(color: Color(0xFF4f545c), width: 2)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            q.displayName ?? q.role,
            style: const TextStyle(color: Color(0xFF8e9297), fontSize: 10, fontWeight: FontWeight.w900),
          ),
          const SizedBox(height: 2),
          Text(
            q.content,
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: const TextStyle(color: Color(0xFFb9bbbe), fontSize: 12, fontStyle: FontStyle.italic),
          ),
        ],
      ),
    );
  }

  Widget _buildThoughts() {
    return GestureDetector(
      onTap: () => setState(() => _thoughtExpanded = !_thoughtExpanded),
      child: Container(
        margin: const EdgeInsets.only(bottom: 12),
        child: ClipRRect(
          borderRadius: BorderRadius.circular(8),
          child: BackdropFilter(
            filter: ImageFilter.blur(sigmaX: 40, sigmaY: 40), // 🎯 Ultra-high liquid blur
            child: Container(
              padding: const EdgeInsets.all(10),
              decoration: BoxDecoration(
                gradient: LinearGradient(
                  begin: Alignment.topLeft,
                  end: Alignment.bottomRight,
                  colors: [
                    const Color(0xFF2f3136).withValues(alpha: 0.5),
                    const Color(0xFF1e293b).withValues(alpha: 0.2),
                  ],
                ),
                borderRadius: BorderRadius.circular(8),
                border: Border.all(color: Colors.blue.withValues(alpha: 0.15)),
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      const Row(
                        children: [
                          Icon(LucideIcons.brain, size: 12, color: Color(0xFF3b82f6)),
                          SizedBox(width: 8),
                          Text(
                            'DEEP THOUGHT',
                            style: TextStyle(
                              color: Color(0xFF3b82f6),
                              fontSize: 8,
                              fontWeight: FontWeight.w900,
                              letterSpacing: 1,
                            ),
                          ),
                        ],
                      ),
                      Icon(
                        _thoughtExpanded ? LucideIcons.chevronUp : LucideIcons.chevronDown,
                        size: 14,
                        color: Colors.white10,
                      ),
                    ],
                  ),
                  if (_thoughtExpanded) ...[
                    const SizedBox(height: 10),
                    Text(
                      widget.message.thought!,
                      style: const TextStyle(
                        color: Color(0xFFb9bbbe),
                        fontSize: 12,
                        fontStyle: FontStyle.italic,
                        height: 1.4,
                      ),
                    ),
                  ],
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}

class CodeElementBuilder extends MarkdownElementBuilder {
  @override
  Widget? visitElementAfter(md.Element element, TextStyle? preferredStyle) {
    var language = 'text';

    if (element.attributes['class'] != null) {
      String lg = element.attributes['class'] as String;
      if (lg.contains('-')) {
        language = lg.split('-')[1];
      }
    }

    return CodeBlock(
      code: element.textContent.trim(),
      language: language,
    );
  }
}

class _AudioPlayerWidget extends StatefulWidget {
  final String url;
  const _AudioPlayerWidget({required this.url});

  @override
  State<_AudioPlayerWidget> createState() => _AudioPlayerWidgetState();
}

class _AudioPlayerWidgetState extends State<_AudioPlayerWidget> {
  final AudioPlayer _player = AudioPlayer();
  bool _isPlaying = false;
  Duration _duration = Duration.zero;
  Duration _position = Duration.zero;

  @override
  void initState() {
    super.initState();
    _player.onDurationChanged.listen((d) => setState(() => _duration = d));
    _player.onPositionChanged.listen((p) => setState(() => _position = p));
    _player.onPlayerStateChanged.listen((s) {
      if (mounted) setState(() => _isPlaying = s == PlayerState.playing);
    });
  }

  @override
  void dispose() {
    _player.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      margin: const EdgeInsets.only(bottom: 8),
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: const Color(0xFF202225),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        children: [
          IconButton(
            icon: Icon(_isPlaying ? LucideIcons.pause : LucideIcons.play, color: Colors.blue),
            onPressed: () {
              if (_isPlaying) {
                _player.pause();
              } else {
                _player.play(UrlSource(widget.url));
              }
            },
          ),
          Expanded(
            child: Slider(
              activeColor: Colors.blue,
              inactiveColor: Colors.white10,
              value: _position.inSeconds.toDouble(),
              max: _duration.inSeconds.toDouble() > 0 ? _duration.inSeconds.toDouble() : 1.0,
              onChanged: (value) {
                _player.seek(Duration(seconds: value.toInt()));
              },
            ),
          ),
          Text(
            '${_position.inMinutes}:${(_position.inSeconds % 60).toString().padLeft(2, '0')}',
            style: const TextStyle(color: Color(0xFF8e9297), fontSize: 10, fontFamily: 'monospace'),
          ),
          const SizedBox(width: 8),
        ],
      ),
    );
  }
}
