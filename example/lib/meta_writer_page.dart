import 'dart:io';
import 'dart:typed_data';


import 'package:audiopc/audiopc.dart';
import 'package:file_picker/file_picker.dart';
import 'package:material_ui/material_ui.dart';

/// A page that lets the user pick an audio file, read its current metadata,
/// edit every tag field (including embedded artwork), and write it back to
/// disk using the Rust `write_metadata` bridge.
class MetaWriterPage extends StatefulWidget {
  const MetaWriterPage({super.key});

  @override
  State<MetaWriterPage> createState() => _MetaWriterPageState();
}

class _MetaWriterPageState extends State<MetaWriterPage> {
  // ── Controllers ───────────────────────────────────────────────────────
  final _titleCtrl = TextEditingController();
  final _artistCtrl = TextEditingController();
  final _albumCtrl = TextEditingController();
  final _yearCtrl = TextEditingController();
  final _genreCtrl = TextEditingController();
  final _trackCtrl = TextEditingController();

  // ── State ─────────────────────────────────────────────────────────────
  String? _audioPath;
  AudioMetadata? _original;
  Uint8List? _artworkBytes;
  String? _artworkMime;
  bool _artworkChanged = false;

  bool _loading = false;
  bool _saving = false;
  String? _statusMessage;
  bool _statusIsError = false;

  @override
  void dispose() {
    _titleCtrl.dispose();
    _artistCtrl.dispose();
    _albumCtrl.dispose();
    _yearCtrl.dispose();
    _genreCtrl.dispose();
    _trackCtrl.dispose();
    super.dispose();
  }

  // ── File pickers ──────────────────────────────────────────────────────

  /// Opens a file picker restricted to audio files and reads existing metadata.
  Future<void> _pickAudioFile() async {
    final result = await FilePicker.pickFiles(type: FileType.audio);
    final path = result?.files.single.path;
    if (path == null) return;

    setState(() {
      _audioPath = path;
      _loading = true;
      _statusMessage = null;
      _artworkChanged = false;
    });

    try {
      final meta = await readMetadata(path: path);
      _populate(meta);
      setState(() {
        _original = meta;
        _artworkBytes = meta.artworkData;
        _artworkMime = meta.artworkMimeType;
        _loading = false;
      });
    } catch (e) {
      _setStatus('Failed to read metadata: $e', isError: true);
      setState(() => _loading = false);
    }
  }

  /// Opens an image file picker and stores the bytes as the new artwork.
  Future<void> _pickArtwork() async {
    final result = await FilePicker.pickFiles(type: FileType.image);
    final file = result?.files.single;
    if (file == null) return;

    final path = file.path;
    if (path == null) return;

    final bytes = await File(path).readAsBytes();
    final ext = file.extension?.toLowerCase() ?? '';
    final mime = ext == 'png' ? 'image/png' : 'image/jpeg';

    setState(() {
      _artworkBytes = bytes;
      _artworkMime = mime;
      _artworkChanged = true;
      _statusMessage = null;
    });
  }

  /// Clears the embedded artwork (sets it to empty bytes on save).
  void _clearArtwork() {
    setState(() {
      _artworkBytes = null;
      _artworkMime = null;
      _artworkChanged = true;
      _statusMessage = null;
    });
  }

  // ── Metadata helpers ──────────────────────────────────────────────────

  /// Fills text controllers from an [AudioMetadata] object.
  void _populate(AudioMetadata meta) {
    _titleCtrl.text = meta.title ?? '';
    _artistCtrl.text = meta.artist ?? '';
    _albumCtrl.text = meta.album ?? '';
    _yearCtrl.text = meta.year ?? '';
    _genreCtrl.text = meta.genre ?? '';
    _trackCtrl.text = meta.trackNumber ?? '';
  }

  /// Builds a [MetadataUpdate] from the current form state.
  MetadataUpdate _buildUpdate() {
    String? text(TextEditingController c) =>
        c.text.trim().isEmpty ? null : c.text.trim();

    // For artwork: if unchanged pass null (leave as-is).
    // If changed, pass the bytes (may be empty to clear).
    Uint8List? artworkData;
    String? artworkMime;
    if (_artworkChanged) {
      artworkData = _artworkBytes ?? Uint8List(0);
      artworkMime = _artworkMime;
    }

    return MetadataUpdate(
      title: text(_titleCtrl),
      artist: text(_artistCtrl),
      album: text(_albumCtrl),
      year: text(_yearCtrl),
      genre: text(_genreCtrl),
      trackNumber: text(_trackCtrl),
      artworkData: artworkData,
      artworkMimeType: artworkMime,
    );
  }

  // ── Save ──────────────────────────────────────────────────────────────

  Future<void> _save() async {
    if (_audioPath == null) return;

    setState(() {
      _saving = true;
      _statusMessage = null;
    });

    try {
      final update = _buildUpdate();
      await writeMetadata(path: _audioPath!, update: update);
      _setStatus('Metadata saved successfully ✓', isError: false);
      // Re-read to confirm what was written.
      final meta = await readMetadata(path: _audioPath!);
      setState(() {
        _original = meta;
        _artworkChanged = false;
      });
    } catch (e) {
      _setStatus('Save failed: $e', isError: true);
    } finally {
      setState(() => _saving = false);
    }
  }

  void _setStatus(String msg, {required bool isError}) {
    setState(() {
      _statusMessage = msg;
      _statusIsError = isError;
    });
  }

  // ── Build ─────────────────────────────────────────────────────────────

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final scheme = theme.colorScheme;

    return Scaffold(
      appBar: AppBar(
        title: const Text('Meta Writer'),
        actions: [
          if (_audioPath != null && !_saving)
            TextButton.icon(
              onPressed: _save,
              icon: const Icon(Icons.save),
              label: const Text('Save'),
            ),
          if (_saving)
            const Padding(
              padding: EdgeInsets.symmetric(horizontal: 16),
              child: Center(
                child: SizedBox(
                  width: 20,
                  height: 20,
                  child: CircularProgressIndicator(strokeWidth: 2),
                ),
              ),
            ),
        ],
      ),
      body: SafeArea(
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // ── File picker card ───────────────────────────────────
              _SectionCard(
                child: Row(
                  children: [
                    Expanded(
                      child: Text(
                        _audioPath != null
                            ? _audioPath!.split('/').last
                            : 'No audio file selected',
                        style: theme.textTheme.bodyMedium?.copyWith(
                          color: _audioPath != null
                              ? scheme.onSurface
                              : scheme.onSurface.withValues(alpha: 0.5),
                        ),
                        overflow: TextOverflow.ellipsis,
                      ),
                    ),
                    const SizedBox(width: 12),
                    FilledButton.icon(
                      onPressed: _loading ? null : _pickAudioFile,
                      icon: _loading
                          ? const SizedBox(
                              width: 16,
                              height: 16,
                              child: CircularProgressIndicator(strokeWidth: 2),
                            )
                          : const Icon(Icons.audio_file, size: 18),
                      label: Text(_audioPath == null ? 'Select File' : 'Change'),
                    ),
                  ],
                ),
              ),

              if (_statusMessage != null) ...[
                const SizedBox(height: 8),
                _StatusBanner(
                  message: _statusMessage!,
                  isError: _statusIsError,
                ),
              ],

              const SizedBox(height: 16),

              // ── Artwork + text fields ──────────────────────────────
              Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  // Artwork column
                  Column(
                    children: [
                      _ArtworkWidget(
                        bytes: _artworkBytes,
                        changed: _artworkChanged,
                      ),
                      const SizedBox(height: 8),
                      Row(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          OutlinedButton.icon(
                            onPressed: _audioPath == null ? null : _pickArtwork,
                            icon: const Icon(Icons.image, size: 16),
                            label: const Text('Set'),
                            style: OutlinedButton.styleFrom(
                              padding: const EdgeInsets.symmetric(
                                horizontal: 10,
                                vertical: 6,
                              ),
                              textStyle: const TextStyle(fontSize: 12),
                            ),
                          ),
                          const SizedBox(width: 6),
                          OutlinedButton.icon(
                            onPressed: (_audioPath == null ||
                                    (_artworkBytes == null && !_artworkChanged))
                                ? null
                                : _clearArtwork,
                            icon: const Icon(Icons.delete_outline, size: 16),
                            label: const Text('Clear'),
                            style: OutlinedButton.styleFrom(
                              padding: const EdgeInsets.symmetric(
                                horizontal: 10,
                                vertical: 6,
                              ),
                              textStyle: const TextStyle(fontSize: 12),
                              foregroundColor: scheme.error,
                            ),
                          ),
                        ],
                      ),
                    ],
                  ),
                  const SizedBox(width: 16),

                  // Text fields column
                  Expanded(
                    child: Column(
                      children: [
                        _MetaField(
                          controller: _titleCtrl,
                          label: 'Title',
                          icon: Icons.music_note,
                          enabled: _audioPath != null,
                        ),
                        const SizedBox(height: 10),
                        _MetaField(
                          controller: _artistCtrl,
                          label: 'Artist',
                          icon: Icons.person,
                          enabled: _audioPath != null,
                        ),
                        const SizedBox(height: 10),
                        _MetaField(
                          controller: _albumCtrl,
                          label: 'Album',
                          icon: Icons.album,
                          enabled: _audioPath != null,
                        ),
                      ],
                    ),
                  ),
                ],
              ),

              const SizedBox(height: 10),

              // ── Second row of fields ───────────────────────────────
              Row(
                children: [
                  Expanded(
                    child: _MetaField(
                      controller: _yearCtrl,
                      label: 'Year',
                      icon: Icons.calendar_today,
                      enabled: _audioPath != null,
                      keyboardType: TextInputType.number,
                    ),
                  ),
                  const SizedBox(width: 10),
                  Expanded(
                    child: _MetaField(
                      controller: _trackCtrl,
                      label: 'Track #',
                      icon: Icons.format_list_numbered,
                      enabled: _audioPath != null,
                      keyboardType: TextInputType.text,
                    ),
                  ),
                  const SizedBox(width: 10),
                  Expanded(
                    flex: 2,
                    child: _MetaField(
                      controller: _genreCtrl,
                      label: 'Genre',
                      icon: Icons.category,
                      enabled: _audioPath != null,
                    ),
                  ),
                ],
              ),

              // ── Original values inspector ──────────────────────────
              if (_original != null) ...[
                const SizedBox(height: 24),
                _OriginalMetaCard(meta: _original!),
              ],
            ],
          ),
        ),
      ),
    );
  }
}

// ─── Helper widgets ────────────────────────────────────────────────────────

/// A card-like container with consistent padding and elevation.
class _SectionCard extends StatelessWidget {
  const _SectionCard({required this.child});

  final Widget child;

  @override
  Widget build(BuildContext context) {
    return Card(
      margin: EdgeInsets.zero,
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
        child: child,
      ),
    );
  }
}

/// Displays the artwork image or a placeholder.
class _ArtworkWidget extends StatelessWidget {
  const _ArtworkWidget({required this.bytes, required this.changed});

  final Uint8List? bytes;
  final bool changed;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;

    return Container(
      width: 120,
      height: 120,
      decoration: BoxDecoration(
        color: scheme.surfaceContainerHighest,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(
          color: changed
              ? scheme.primary.withValues(alpha: 0.6)
              : scheme.outlineVariant,
          width: changed ? 2 : 1,
        ),
      ),
      clipBehavior: Clip.antiAlias,
      child: bytes != null && bytes!.isNotEmpty
          ? Image.memory(bytes!, fit: BoxFit.cover)
          : Icon(
              Icons.image_not_supported_outlined,
              size: 48,
              color: scheme.onSurface.withValues(alpha: 0.3),
            ),
    );
  }
}

/// A labelled text field for one metadata value.
class _MetaField extends StatelessWidget {
  const _MetaField({
    required this.controller,
    required this.label,
    required this.icon,
    this.enabled = true,
    this.keyboardType,
  });

  final TextEditingController controller;
  final String label;
  final IconData icon;
  final bool enabled;
  final TextInputType? keyboardType;

  @override
  Widget build(BuildContext context) {
    return TextField(
      controller: controller,
      enabled: enabled,
      keyboardType: keyboardType,
      decoration: InputDecoration(
        labelText: label,
        prefixIcon: Icon(icon, size: 18),
        border: OutlineInputBorder(borderRadius: BorderRadius.circular(10)),
        isDense: true,
        contentPadding: const EdgeInsets.symmetric(
          vertical: 10,
          horizontal: 12,
        ),
      ),
    );
  }
}

/// A compact status banner shown below the file picker.
class _StatusBanner extends StatelessWidget {
  const _StatusBanner({required this.message, required this.isError});

  final String message;
  final bool isError;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final color = isError ? scheme.error : scheme.primary;

    return Container(
      width: double.infinity,
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.1),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: color.withValues(alpha: 0.4)),
      ),
      child: Row(
        children: [
          Icon(
            isError ? Icons.error_outline : Icons.check_circle_outline,
            size: 16,
            color: color,
          ),
          const SizedBox(width: 8),
          Expanded(
            child: Text(
              message,
              style: TextStyle(color: color, fontSize: 13),
            ),
          ),
        ],
      ),
    );
  }
}

/// Collapsible card showing the originally read metadata for reference.
class _OriginalMetaCard extends StatefulWidget {
  const _OriginalMetaCard({required this.meta});

  final AudioMetadata meta;

  @override
  State<_OriginalMetaCard> createState() => _OriginalMetaCardState();
}

class _OriginalMetaCardState extends State<_OriginalMetaCard> {
  bool _expanded = false;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final meta = widget.meta;

    final rows = <(String, String?)>[
      ('Title', meta.title),
      ('Artist', meta.artist),
      ('Album', meta.album),
      ('Year', meta.year),
      ('Genre', meta.genre),
      ('Track #', meta.trackNumber),
      ('Duration', _fmtMs(meta.durationMs?.toInt())),
      ('Bitrate', meta.bitrateKbps != null ? '${meta.bitrateKbps} kbps' : null),
      (
        'Sample rate',
        meta.sampleRateHz != null ? '${meta.sampleRateHz} Hz' : null,
      ),
      ('Channels', meta.channels?.toString()),
      ('Artwork MIME', meta.artworkMimeType),
    ];

    return _SectionCard(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          InkWell(
            onTap: () => setState(() => _expanded = !_expanded),
            borderRadius: BorderRadius.circular(6),
            child: Row(
              children: [
                Icon(Icons.info_outline, size: 18),
                const SizedBox(width: 8),
                Text(
                  'Original metadata',
                  style: theme.textTheme.titleSmall,
                ),
                const Spacer(),
                Icon(
                  _expanded ? Icons.expand_less : Icons.expand_more,
                  size: 20,
                ),
              ],
            ),
          ),
          if (_expanded) ...[
            const SizedBox(height: 10),
            const Divider(height: 1),
            const SizedBox(height: 8),
            ...rows.map(
              (r) => Padding(
                padding: const EdgeInsets.symmetric(vertical: 2),
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    SizedBox(
                      width: 100,
                      child: Text(
                        '${r.$1}:',
                        style: theme.textTheme.bodySmall?.copyWith(
                          color: theme.colorScheme.onSurface.withValues(
                            alpha: 0.6,
                          ),
                        ),
                      ),
                    ),
                    Expanded(
                      child: Text(
                        r.$2 ?? '—',
                        style: theme.textTheme.bodySmall,
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ],
        ],
      ),
    );
  }

  String? _fmtMs(int? ms) {
    if (ms == null || ms < 0) return null;
    final s = ms ~/ 1000;
    final m = s ~/ 60;
    final sec = s % 60;
    return '${m.toString().padLeft(2, '0')}:${sec.toString().padLeft(2, '0')}';
  }
}
