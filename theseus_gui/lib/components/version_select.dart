import 'package:flutter/material.dart';
import 'package:theseus_gui/client_provider.dart';
import 'package:theseus_gui/generated/theseus.pbgrpc.dart';

class VersionSelect extends StatefulWidget {
  final Function(String) onSelect;
  VersionSelect(this.onSelect);

  @override
  State<StatefulWidget> createState() => _VersionSelectState();
}

class _VersionSelectState extends State<VersionSelect> {
  int radioValue = 0;
  List<String> versions = [];
  ScrollController scrollController = ScrollController();

  setRadioValue(radioValue) {
    setState(() {
      this.radioValue = radioValue;
    });
  }

  setVersions(versions) {
    setState(() {
      this.versions = versions;
    });
  }

  @override
  void initState() {
    super.initState();
    client().then((client) {
      client.getCatalogue(Empty()).then((catalogue) {
        setVersions(catalogue.catalogue.map((e) => e.id).toList());
      });
    });
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      child: ListView.builder(
        itemCount: versions.length,
        itemBuilder: (ctx, ind) {
          return RadioListTile(
            value: ind,
            groupValue: radioValue,
            onChanged: (ind) {
              setRadioValue(ind);
              widget.onSelect(versions[ind as int]);
            },
            title: Text(versions[ind]),
          );
        },
        controller: scrollController,
      ),
    );
  }
}
