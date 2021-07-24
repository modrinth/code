import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:theseus_gui/client_provider.dart';
import 'package:theseus_gui/generated/theseus.pbgrpc.dart' as theseus;

class Task extends StatefulWidget {
  final Int64 id;
  final String name;
  Task(this.id, this.name);

  @override
  State<StatefulWidget> createState() => _TaskState();
}

class _TaskState extends State<Task> {
  int finished = 0;
  int total = 1;
  String message = "";

  setProgress(progress) {
    setState(() {
      finished = progress.finished.toInt();
      total = progress.total.toInt();
      message = progress.message;
    });
  }

  @override
  void initState() {
    super.initState();
    client().then((client) => client
        .streamTaskProgress(
          theseus.Task(
            id: widget.id,
            name: widget.name,
          ),
        )
        .listen(setProgress));
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text("[" +
              widget.name +
              "] [" +
              finished.toString() +
              "/" +
              total.toString() +
              "] " +
              message),
          LinearProgressIndicator(
            value: finished / total,
            minHeight: 12,
          )
        ],
      ),
    );
  }
}
