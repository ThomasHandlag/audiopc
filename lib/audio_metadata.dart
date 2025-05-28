import 'package:flutter/foundation.dart';

class AudioMetaData {
  AudioMetaData(
      {this.title,
      this.artist,
      this.albumTitle,
      this.albumArtist,
      this.genre,
      this.timeReleased,
      required this.thumbnail,
      this.copyRight});

  final String? title;
  final String? artist;
  final String? albumTitle;
  final String? albumArtist;
  final String? genre;
  final String? timeReleased;
  final Uint8List? thumbnail;
  final String? copyRight;
  factory AudioMetaData.fromMap(Map<String, dynamic> map) {
    final rawData = map['artwork'] ?? Uint8List(0);
    int jpegStart = rawData.indexOf(255);
    Uint8List? imageData;
    if (jpegStart != -1 && rawData[jpegStart + 1] == 216) {
       imageData = Uint8List.fromList(rawData.sublist(jpegStart));
    }

    final title = removeTerminator(map['title']);
    final artist = removeTerminator(map['artist']);
    final albumTitle = removeTerminator(map['albumTitle']);
    final albumArtist = removeTerminator(map['albumArtist']);
    final genre = removeTerminator(map['genre']);
    final timeReleased = map['timeReleased'] ?? "";
    final copyRight = removeTerminator(map['copyRight']);

    return AudioMetaData(
        title: title,
        artist: artist,
        albumTitle: albumTitle,
        albumArtist: albumArtist,
        genre: genre,
        thumbnail: imageData,
        timeReleased: timeReleased,
        copyRight: copyRight);
  }

  static String removeTerminator(String? str) {
    if (str == null) {
      return '';
    }
    return str.substring(0, !str.contains('\x00') ? str.length : str.indexOf('\x00'));
  }

  @override
  String toString() {
    return 'AudioMetaData{title: $title, artist: $artist, albumTitle: $albumTitle, albumArtist: $albumArtist, genre: $genre, timeReleased: $timeReleased, copyRight: $copyRight}';
  }
}
