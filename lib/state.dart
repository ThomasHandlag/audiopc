import 'dart:async';

enum PlayerState { idle, loading, playing, paused, error }

mixin class StateMixin {
  PlayerState state = PlayerState.idle;

  void setState(PlayerState newState) {
    state = newState;
    stateStreamController.add(state);
  }

  final StreamController<PlayerState> stateStreamController =
      StreamController.broadcast();

  Stream<PlayerState> get stateStream => stateStreamController.stream;
}
