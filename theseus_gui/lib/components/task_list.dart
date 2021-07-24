import 'package:flutter/material.dart';
import 'package:theseus_gui/client_provider.dart';
import 'package:theseus_gui/components/task.dart';
import 'package:theseus_gui/generated/theseus.pbgrpc.dart' as theseus;

List _tasks = [];
List _listeners = [];

void init() {
  client().then((client) {
    client.getTaskList(theseus.Empty()).then((taskList) {
      _listeners.forEach((listener) {
        taskList.taskList.forEach((task) => listener(task));
      });
    });
  });
}

void addTask(task) {
  _listeners.forEach((listener) => listener(task));
}

class TaskList extends StatefulWidget {
  @override
  State<StatefulWidget> createState() => _TaskListState();
}

class _TaskListState extends State {
  ScrollController scrollController = ScrollController();

  addTask(task) {
    setState(() {
      _tasks.add(task);
    });
  }

  @override
  void initState() {
    super.initState();
    _listeners.add(addTask);
  }

  @override
  Widget build(BuildContext context) {
    return ListView.builder(
      itemCount: _tasks.length,
      itemBuilder: (ctx, ind) {
        theseus.Task task = _tasks[ind];
        return Task(task.id, task.name);
      },
      controller: scrollController,
    );
  }
}
