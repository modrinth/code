///
//  Generated code. Do not modify.
//  source: theseus.proto
//
// @dart = 2.12
// ignore_for_file: annotate_overrides,camel_case_types,unnecessary_const,non_constant_identifier_names,library_prefixes,unused_import,unused_shown_name,return_of_invalid_type,unnecessary_this,prefer_final_fields,deprecated_member_use_from_same_package

import 'dart:core' as $core;
import 'dart:convert' as $convert;
import 'dart:typed_data' as $typed_data;
@$core.Deprecated('Use emptyDescriptor instead')
const Empty$json = const {
  '1': 'Empty',
};

/// Descriptor for `Empty`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List emptyDescriptor = $convert.base64Decode('CgVFbXB0eQ==');
@$core.Deprecated('Use catalogueDescriptor instead')
const Catalogue$json = const {
  '1': 'Catalogue',
  '2': const [
    const {'1': 'catalogue', '3': 1, '4': 3, '5': 11, '6': '.theseus.Version', '10': 'catalogue'},
  ],
};

/// Descriptor for `Catalogue`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List catalogueDescriptor = $convert.base64Decode('CglDYXRhbG9ndWUSLgoJY2F0YWxvZ3VlGAEgAygLMhAudGhlc2V1cy5WZXJzaW9uUgljYXRhbG9ndWU=');
@$core.Deprecated('Use versionDescriptor instead')
const Version$json = const {
  '1': 'Version',
  '2': const [
    const {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
  ],
};

/// Descriptor for `Version`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List versionDescriptor = $convert.base64Decode('CgdWZXJzaW9uEg4KAmlkGAEgASgJUgJpZA==');
@$core.Deprecated('Use launchOptionsDescriptor instead')
const LaunchOptions$json = const {
  '1': 'LaunchOptions',
  '2': const [
    const {'1': 'version_id', '3': 1, '4': 1, '5': 9, '10': 'versionId'},
    const {'1': 'username', '3': 2, '4': 1, '5': 9, '10': 'username'},
  ],
};

/// Descriptor for `LaunchOptions`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List launchOptionsDescriptor = $convert.base64Decode('Cg1MYXVuY2hPcHRpb25zEh0KCnZlcnNpb25faWQYASABKAlSCXZlcnNpb25JZBIaCgh1c2VybmFtZRgCIAEoCVIIdXNlcm5hbWU=');
@$core.Deprecated('Use taskDescriptor instead')
const Task$json = const {
  '1': 'Task',
  '2': const [
    const {'1': 'id', '3': 1, '4': 1, '5': 4, '10': 'id'},
    const {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
  ],
};

/// Descriptor for `Task`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List taskDescriptor = $convert.base64Decode('CgRUYXNrEg4KAmlkGAEgASgEUgJpZBISCgRuYW1lGAIgASgJUgRuYW1l');
@$core.Deprecated('Use taskListDescriptor instead')
const TaskList$json = const {
  '1': 'TaskList',
  '2': const [
    const {'1': 'task_list', '3': 1, '4': 3, '5': 11, '6': '.theseus.Task', '10': 'taskList'},
  ],
};

/// Descriptor for `TaskList`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List taskListDescriptor = $convert.base64Decode('CghUYXNrTGlzdBIqCgl0YXNrX2xpc3QYASADKAsyDS50aGVzZXVzLlRhc2tSCHRhc2tMaXN0');
@$core.Deprecated('Use taskProgressDescriptor instead')
const TaskProgress$json = const {
  '1': 'TaskProgress',
  '2': const [
    const {'1': 'finished', '3': 1, '4': 1, '5': 4, '10': 'finished'},
    const {'1': 'total', '3': 2, '4': 1, '5': 4, '10': 'total'},
    const {'1': 'message', '3': 3, '4': 1, '5': 9, '9': 0, '10': 'message', '17': true},
  ],
  '8': const [
    const {'1': '_message'},
  ],
};

/// Descriptor for `TaskProgress`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List taskProgressDescriptor = $convert.base64Decode('CgxUYXNrUHJvZ3Jlc3MSGgoIZmluaXNoZWQYASABKARSCGZpbmlzaGVkEhQKBXRvdGFsGAIgASgEUgV0b3RhbBIdCgdtZXNzYWdlGAMgASgJSABSB21lc3NhZ2WIAQFCCgoIX21lc3NhZ2U=');
