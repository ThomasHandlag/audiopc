import 'package:code_assets/code_assets.dart';
import 'package:hooks/hooks.dart';
import 'package:native_toolchain_rust/native_toolchain_rust.dart';

void main(List<String> args) async {
  await build(args, (input, output) async {
    if (!input.config.buildCodeAssets) {
      return;
    }

    await const RustBuilder(
      assetName: 'audiopc.g.dart',
      cratePath: 'rust',
    ).run(input: input, output: output);
  });
}