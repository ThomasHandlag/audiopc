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

class StateEvent extends PlayerEvent {
  StateEvent({required double super.value})
      : super(type: PlayerEventType.state);
}

class ErrorEvent extends PlayerEvent {
  ErrorEvent({required String super.value})
      : super(type: PlayerEventType.error);
}

class NoneEvent extends PlayerEvent {
  NoneEvent() : super(type: PlayerEventType.none, value: null);
}

class DurationEvent extends PlayerEvent {
  DurationEvent({required double super.value})
      : super(type: PlayerEventType.duration);
}

sealed class PlayerEventType {
  static const state = 'state';
  static const error = 'error';
  static const duration = 'duration';
  static const none = 'none';
}
