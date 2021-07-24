import 'package:flutter/material.dart';
import 'package:theseus_gui/client_provider.dart' as client_provider;
import 'package:theseus_gui/components/task_list.dart' as task_list;
import 'package:theseus_gui/routes/default.dart';

void main() {
  client_provider.init();
  task_list.init();
  runApp(MyApp());
}

class MyApp extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      home: DefaultRoute(),
    );
  }
}
