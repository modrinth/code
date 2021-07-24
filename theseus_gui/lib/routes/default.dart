import 'package:flutter/material.dart';
import 'package:theseus_gui/client_provider.dart';
import 'package:theseus_gui/components/version_select.dart';
import 'package:theseus_gui/components/task_list.dart';
import 'package:theseus_gui/generated/theseus.pbgrpc.dart' as theseus;

String versionId = "1.16.5";
String username = "default";

class DefaultRoute extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Row(
        children: [
          Expanded(
            flex: 1,
            child: Column(
              children: [
                Expanded(child: VersionSelect((id) {
                  versionId = id;
                })),
                TextField(
                  onChanged: (value) {
                    username = value;
                  },
                  decoration: InputDecoration(
                    hintText: "Username",
                    contentPadding: EdgeInsets.all(10),
                  ),
                ),
              ],
            ),
          ),
          Expanded(
            flex: 2,
            child: Column(
              children: [
                Expanded(child: TaskList()),
                Row(
                  children: [
                    Expanded(
                      child: TextButton(
                        onPressed: () {
                          client().then((client) {
                            client
                                .launch(
                                  theseus.LaunchOptions(
                                    versionId: versionId,
                                    username: username,
                                  ),
                                )
                                .then(addTask);
                          });
                        },
                        child: Text("Launch"),
                        style: TextButton.styleFrom(
                            fixedSize: Size.fromHeight(48)),
                      ),
                    )
                  ],
                ),
              ],
            ),
          )
        ],
      ),
    );
  }
}
