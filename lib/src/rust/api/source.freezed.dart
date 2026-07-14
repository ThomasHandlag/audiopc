// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'source.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AudioSource {

 Object get field0;



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AudioSource&&const DeepCollectionEquality().equals(other.field0, field0));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(field0));

@override
String toString() {
  return 'AudioSource(field0: $field0)';
}


}

/// @nodoc
class $AudioSourceCopyWith<$Res>  {
$AudioSourceCopyWith(AudioSource _, $Res Function(AudioSource) __);
}


/// Adds pattern-matching-related methods to [AudioSource].
extension AudioSourcePatterns on AudioSource {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( AudioSource_Path value)?  path,TResult Function( AudioSource_Url value)?  url,TResult Function( AudioSource_Memory value)?  memory,required TResult orElse(),}){
final _that = this;
switch (_that) {
case AudioSource_Path() when path != null:
return path(_that);case AudioSource_Url() when url != null:
return url(_that);case AudioSource_Memory() when memory != null:
return memory(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( AudioSource_Path value)  path,required TResult Function( AudioSource_Url value)  url,required TResult Function( AudioSource_Memory value)  memory,}){
final _that = this;
switch (_that) {
case AudioSource_Path():
return path(_that);case AudioSource_Url():
return url(_that);case AudioSource_Memory():
return memory(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( AudioSource_Path value)?  path,TResult? Function( AudioSource_Url value)?  url,TResult? Function( AudioSource_Memory value)?  memory,}){
final _that = this;
switch (_that) {
case AudioSource_Path() when path != null:
return path(_that);case AudioSource_Url() when url != null:
return url(_that);case AudioSource_Memory() when memory != null:
return memory(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String field0)?  path,TResult Function( String field0)?  url,TResult Function( Uint8List field0)?  memory,required TResult orElse(),}) {final _that = this;
switch (_that) {
case AudioSource_Path() when path != null:
return path(_that.field0);case AudioSource_Url() when url != null:
return url(_that.field0);case AudioSource_Memory() when memory != null:
return memory(_that.field0);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String field0)  path,required TResult Function( String field0)  url,required TResult Function( Uint8List field0)  memory,}) {final _that = this;
switch (_that) {
case AudioSource_Path():
return path(_that.field0);case AudioSource_Url():
return url(_that.field0);case AudioSource_Memory():
return memory(_that.field0);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String field0)?  path,TResult? Function( String field0)?  url,TResult? Function( Uint8List field0)?  memory,}) {final _that = this;
switch (_that) {
case AudioSource_Path() when path != null:
return path(_that.field0);case AudioSource_Url() when url != null:
return url(_that.field0);case AudioSource_Memory() when memory != null:
return memory(_that.field0);case _:
  return null;

}
}

}

/// @nodoc


class AudioSource_Path extends AudioSource {
  const AudioSource_Path(this.field0): super._();
  

@override final  String field0;

/// Create a copy of AudioSource
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AudioSource_PathCopyWith<AudioSource_Path> get copyWith => _$AudioSource_PathCopyWithImpl<AudioSource_Path>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AudioSource_Path&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'AudioSource.path(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $AudioSource_PathCopyWith<$Res> implements $AudioSourceCopyWith<$Res> {
  factory $AudioSource_PathCopyWith(AudioSource_Path value, $Res Function(AudioSource_Path) _then) = _$AudioSource_PathCopyWithImpl;
@useResult
$Res call({
 String field0
});




}
/// @nodoc
class _$AudioSource_PathCopyWithImpl<$Res>
    implements $AudioSource_PathCopyWith<$Res> {
  _$AudioSource_PathCopyWithImpl(this._self, this._then);

  final AudioSource_Path _self;
  final $Res Function(AudioSource_Path) _then;

/// Create a copy of AudioSource
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(AudioSource_Path(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AudioSource_Url extends AudioSource {
  const AudioSource_Url(this.field0): super._();
  

@override final  String field0;

/// Create a copy of AudioSource
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AudioSource_UrlCopyWith<AudioSource_Url> get copyWith => _$AudioSource_UrlCopyWithImpl<AudioSource_Url>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AudioSource_Url&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'AudioSource.url(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $AudioSource_UrlCopyWith<$Res> implements $AudioSourceCopyWith<$Res> {
  factory $AudioSource_UrlCopyWith(AudioSource_Url value, $Res Function(AudioSource_Url) _then) = _$AudioSource_UrlCopyWithImpl;
@useResult
$Res call({
 String field0
});




}
/// @nodoc
class _$AudioSource_UrlCopyWithImpl<$Res>
    implements $AudioSource_UrlCopyWith<$Res> {
  _$AudioSource_UrlCopyWithImpl(this._self, this._then);

  final AudioSource_Url _self;
  final $Res Function(AudioSource_Url) _then;

/// Create a copy of AudioSource
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(AudioSource_Url(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AudioSource_Memory extends AudioSource {
  const AudioSource_Memory(this.field0): super._();
  

@override final  Uint8List field0;

/// Create a copy of AudioSource
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AudioSource_MemoryCopyWith<AudioSource_Memory> get copyWith => _$AudioSource_MemoryCopyWithImpl<AudioSource_Memory>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AudioSource_Memory&&const DeepCollectionEquality().equals(other.field0, field0));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(field0));

@override
String toString() {
  return 'AudioSource.memory(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $AudioSource_MemoryCopyWith<$Res> implements $AudioSourceCopyWith<$Res> {
  factory $AudioSource_MemoryCopyWith(AudioSource_Memory value, $Res Function(AudioSource_Memory) _then) = _$AudioSource_MemoryCopyWithImpl;
@useResult
$Res call({
 Uint8List field0
});




}
/// @nodoc
class _$AudioSource_MemoryCopyWithImpl<$Res>
    implements $AudioSource_MemoryCopyWith<$Res> {
  _$AudioSource_MemoryCopyWithImpl(this._self, this._then);

  final AudioSource_Memory _self;
  final $Res Function(AudioSource_Memory) _then;

/// Create a copy of AudioSource
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(AudioSource_Memory(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as Uint8List,
  ));
}


}

// dart format on
