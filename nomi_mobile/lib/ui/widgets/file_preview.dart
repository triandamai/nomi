import 'package:flutter/material.dart';
import 'package:audioplayers/audioplayers.dart';
import 'package:video_player/video_player.dart';
import 'package:cached_network_image/cached_network_image.dart';
import 'package:nomi_mobile/core/config.dart';

class FilePreviewWidget extends StatefulWidget {
  final String url;
  final String mimeType;

  const FilePreviewWidget({super.key, required this.url, required this.mimeType});

  @override
  State<FilePreviewWidget> createState() => _FilePreviewWidgetState();
}

class _FilePreviewWidgetState extends State<FilePreviewWidget> {
  VideoPlayerController? _videoController;
  AudioPlayer? _audioPlayer;

  @override
  void initState() {
    super.initState();
    if (widget.mimeType.startsWith('video/')) {
      _videoController = VideoPlayerController.networkUrl(Uri.parse(widget.url))
        ..initialize().then((_) => setState(() {}));
    } else if (widget.mimeType.startsWith('audio/')) {
      _audioPlayer = AudioPlayer();
      _audioPlayer?.play(UrlSource(widget.url));
    }
  }

  @override
  void dispose() {
    _videoController?.dispose();
    _audioPlayer?.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    if (widget.mimeType.startsWith('image/')) {
      return CachedNetworkImage(imageUrl: widget.url, placeholder: (_, __) => const CircularProgressIndicator());
    } else if (widget.mimeType.startsWith('video/')) {
      return _videoController?.value.isInitialized ?? false
          ? AspectRatio(aspectRatio: _videoController!.value.aspectRatio, child: VideoPlayer(_videoController!))
          : const CircularProgressIndicator();
    } else if (widget.mimeType.startsWith('audio/')) {
      return const Icon(Icons.music_note, size: 64, color: Colors.blue);
    }
    return const Center(child: Text('Unsupported file format', style: TextStyle(color: Colors.white)));
  }
}
