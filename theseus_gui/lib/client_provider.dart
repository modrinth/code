import 'dart:async';
import 'dart:io';
import 'package:flutter/foundation.dart';
import 'package:theseus_gui/generated/theseus.pbgrpc.dart';
import 'package:grpc/grpc.dart';

late TheseusClient _client;
bool _clientInitialized = false;
Completer<TheseusClient> _completer = Completer();

void init() {
  final execPath =
      "../target/" + (kReleaseMode ? "release" : "debug") + "/theseus_daemon";
  Process.start(execPath, [], mode: ProcessStartMode.detached).then((_) {
    _client = TheseusClient(
      ClientChannel(
        "localhost",
        port: 1234,
        options: ChannelOptions(
          credentials: ChannelCredentials.insecure(),
        ),
      ),
    );
    _clientInitialized = true;
    _completer.complete(_client);
  });
}

Future<TheseusClient> client() async {
  if (_clientInitialized) {
    return _client;
  } else {
    return _completer.future;
  }
}
