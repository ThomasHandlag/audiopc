// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'state.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$PlaybackState {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PlaybackState);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PlaybackState()';
}


}

/// @nodoc
class $PlaybackStateCopyWith<$Res>  {
$PlaybackStateCopyWith(PlaybackState _, $Res Function(PlaybackState) __);
}


/// Adds pattern-matching-related methods to [PlaybackState].
extension PlaybackStatePatterns on PlaybackState {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( PlaybackState_Idle value)?  idle,TResult Function( PlaybackState_Buffering value)?  buffering,TResult Function( PlaybackState_Playing value)?  playing,TResult Function( PlaybackState_Paused value)?  paused,TResult Function( PlaybackState_Stopped value)?  stopped,TResult Function( PlaybackState_Completed value)?  completed,TResult Function( PlaybackState_Error value)?  error,required TResult orElse(),}){
final _that = this;
switch (_that) {
case PlaybackState_Idle() when idle != null:
return idle(_that);case PlaybackState_Buffering() when buffering != null:
return buffering(_that);case PlaybackState_Playing() when playing != null:
return playing(_that);case PlaybackState_Paused() when paused != null:
return paused(_that);case PlaybackState_Stopped() when stopped != null:
return stopped(_that);case PlaybackState_Completed() when completed != null:
return completed(_that);case PlaybackState_Error() when error != null:
return error(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( PlaybackState_Idle value)  idle,required TResult Function( PlaybackState_Buffering value)  buffering,required TResult Function( PlaybackState_Playing value)  playing,required TResult Function( PlaybackState_Paused value)  paused,required TResult Function( PlaybackState_Stopped value)  stopped,required TResult Function( PlaybackState_Completed value)  completed,required TResult Function( PlaybackState_Error value)  error,}){
final _that = this;
switch (_that) {
case PlaybackState_Idle():
return idle(_that);case PlaybackState_Buffering():
return buffering(_that);case PlaybackState_Playing():
return playing(_that);case PlaybackState_Paused():
return paused(_that);case PlaybackState_Stopped():
return stopped(_that);case PlaybackState_Completed():
return completed(_that);case PlaybackState_Error():
return error(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( PlaybackState_Idle value)?  idle,TResult? Function( PlaybackState_Buffering value)?  buffering,TResult? Function( PlaybackState_Playing value)?  playing,TResult? Function( PlaybackState_Paused value)?  paused,TResult? Function( PlaybackState_Stopped value)?  stopped,TResult? Function( PlaybackState_Completed value)?  completed,TResult? Function( PlaybackState_Error value)?  error,}){
final _that = this;
switch (_that) {
case PlaybackState_Idle() when idle != null:
return idle(_that);case PlaybackState_Buffering() when buffering != null:
return buffering(_that);case PlaybackState_Playing() when playing != null:
return playing(_that);case PlaybackState_Paused() when paused != null:
return paused(_that);case PlaybackState_Stopped() when stopped != null:
return stopped(_that);case PlaybackState_Completed() when completed != null:
return completed(_that);case PlaybackState_Error() when error != null:
return error(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  idle,TResult Function()?  buffering,TResult Function()?  playing,TResult Function()?  paused,TResult Function()?  stopped,TResult Function()?  completed,TResult Function( String field0)?  error,required TResult orElse(),}) {final _that = this;
switch (_that) {
case PlaybackState_Idle() when idle != null:
return idle();case PlaybackState_Buffering() when buffering != null:
return buffering();case PlaybackState_Playing() when playing != null:
return playing();case PlaybackState_Paused() when paused != null:
return paused();case PlaybackState_Stopped() when stopped != null:
return stopped();case PlaybackState_Completed() when completed != null:
return completed();case PlaybackState_Error() when error != null:
return error(_that.field0);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  idle,required TResult Function()  buffering,required TResult Function()  playing,required TResult Function()  paused,required TResult Function()  stopped,required TResult Function()  completed,required TResult Function( String field0)  error,}) {final _that = this;
switch (_that) {
case PlaybackState_Idle():
return idle();case PlaybackState_Buffering():
return buffering();case PlaybackState_Playing():
return playing();case PlaybackState_Paused():
return paused();case PlaybackState_Stopped():
return stopped();case PlaybackState_Completed():
return completed();case PlaybackState_Error():
return error(_that.field0);}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  idle,TResult? Function()?  buffering,TResult? Function()?  playing,TResult? Function()?  paused,TResult? Function()?  stopped,TResult? Function()?  completed,TResult? Function( String field0)?  error,}) {final _that = this;
switch (_that) {
case PlaybackState_Idle() when idle != null:
return idle();case PlaybackState_Buffering() when buffering != null:
return buffering();case PlaybackState_Playing() when playing != null:
return playing();case PlaybackState_Paused() when paused != null:
return paused();case PlaybackState_Stopped() when stopped != null:
return stopped();case PlaybackState_Completed() when completed != null:
return completed();case PlaybackState_Error() when error != null:
return error(_that.field0);case _:
  return null;

}
}

}

/// @nodoc


class PlaybackState_Idle extends PlaybackState {
  const PlaybackState_Idle(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PlaybackState_Idle);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PlaybackState.idle()';
}


}




/// @nodoc


class PlaybackState_Buffering extends PlaybackState {
  const PlaybackState_Buffering(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PlaybackState_Buffering);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PlaybackState.buffering()';
}


}




/// @nodoc


class PlaybackState_Playing extends PlaybackState {
  const PlaybackState_Playing(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PlaybackState_Playing);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PlaybackState.playing()';
}


}




/// @nodoc


class PlaybackState_Paused extends PlaybackState {
  const PlaybackState_Paused(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PlaybackState_Paused);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PlaybackState.paused()';
}


}




/// @nodoc


class PlaybackState_Stopped extends PlaybackState {
  const PlaybackState_Stopped(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PlaybackState_Stopped);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PlaybackState.stopped()';
}


}




/// @nodoc


class PlaybackState_Completed extends PlaybackState {
  const PlaybackState_Completed(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PlaybackState_Completed);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PlaybackState.completed()';
}


}




/// @nodoc


class PlaybackState_Error extends PlaybackState {
  const PlaybackState_Error(this.field0): super._();
  

 final  String field0;

/// Create a copy of PlaybackState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PlaybackState_ErrorCopyWith<PlaybackState_Error> get copyWith => _$PlaybackState_ErrorCopyWithImpl<PlaybackState_Error>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PlaybackState_Error&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'PlaybackState.error(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $PlaybackState_ErrorCopyWith<$Res> implements $PlaybackStateCopyWith<$Res> {
  factory $PlaybackState_ErrorCopyWith(PlaybackState_Error value, $Res Function(PlaybackState_Error) _then) = _$PlaybackState_ErrorCopyWithImpl;
@useResult
$Res call({
 String field0
});




}
/// @nodoc
class _$PlaybackState_ErrorCopyWithImpl<$Res>
    implements $PlaybackState_ErrorCopyWith<$Res> {
  _$PlaybackState_ErrorCopyWithImpl(this._self, this._then);

  final PlaybackState_Error _self;
  final $Res Function(PlaybackState_Error) _then;

/// Create a copy of PlaybackState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(PlaybackState_Error(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
