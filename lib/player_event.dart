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

class DurationEvent extends PlayerEvent {
   DurationEvent({required double super.value}) : super(type: PlayerEventType.duration);
}

class StateEvent extends PlayerEvent {
   StateEvent({required double super.value}) : super(type: PlayerEventType.position);
}

class CompletedEvent extends PlayerEvent {
   CompletedEvent({required bool super.value}) : super(type: PlayerEventType.completed);
}

sealed class PlayerEventType {
  static const duration = 'duration';
  static const position = 'state'; 
  static const completed = 'completed';
}


