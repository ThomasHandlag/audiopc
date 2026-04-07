import 'dart:io';

// ignore_for_file: depend_on_referenced_packages

import 'package:logging/logging.dart';
import 'package:native_toolchain_rust/native_toolchain_rust.dart';
import 'package:hooks/hooks.dart';

void main(List<String> args) async {
  if (!_hasConfigArgument(args)) {
    stderr.writeln('Usage: dart run hook/build.dart --config <path-to-input.json>');
    stderr.writeln(
      'For Flutter hook debugging, use the input.json path printed by the build hook logs.',
    );
    exit(64);
  }

  await build(args, (input, output) async {
    final packageName = input.packageName;
    final rustBuilder = RustBuilder(
      cratePath: 'rust_backend',
      assetName: 'src/$packageName.g.dart',
      buildMode: BuildMode.release,
    );

    await rustBuilder.run(
      input: input,
      output: output,
      logger: Logger("RustBuilder"),
    );
  });
}

bool _hasConfigArgument(List<String> args) {
  for (var i = 0; i < args.length; i++) {
    final argument = args[i];
    if (argument.startsWith('--config=')) {
      return argument.length > '--config='.length;
    }
    if (argument == '--config') {
      return i + 1 < args.length && args[i + 1].isNotEmpty;
    }
  }
  return false;
}
