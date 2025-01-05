abstract class PlayerEvent {
  final String type;
  final dynamic value;
  PlayerEvent({required this.type, required this.value});
  Map<String, dynamic> toMap() {
    return {
      'name': type,
      'value': value,
    };
  }
  @override
  String toString() => 'PlayerEvent(name: $type, value: $value)';
}

sealed class PlayerEventType {
  static const duration = 'duration';
  static const position = 'state'; 
  static const completed = 'completed';
}


