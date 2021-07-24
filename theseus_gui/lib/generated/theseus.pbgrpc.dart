///
//  Generated code. Do not modify.
//  source: theseus.proto
//
// @dart = 2.12
// ignore_for_file: annotate_overrides,camel_case_types,unnecessary_const,non_constant_identifier_names,library_prefixes,unused_import,unused_shown_name,return_of_invalid_type,unnecessary_this,prefer_final_fields

import 'dart:async' as $async;

import 'dart:core' as $core;

import 'package:grpc/service_api.dart' as $grpc;
import 'theseus.pb.dart' as $0;
export 'theseus.pb.dart';

class TheseusClient extends $grpc.Client {
  static final _$getCatalogue = $grpc.ClientMethod<$0.Empty, $0.Catalogue>(
      '/theseus.Theseus/GetCatalogue',
      ($0.Empty value) => value.writeToBuffer(),
      ($core.List<$core.int> value) => $0.Catalogue.fromBuffer(value));
  static final _$launch = $grpc.ClientMethod<$0.LaunchOptions, $0.Task>(
      '/theseus.Theseus/Launch',
      ($0.LaunchOptions value) => value.writeToBuffer(),
      ($core.List<$core.int> value) => $0.Task.fromBuffer(value));
  static final _$getTaskList = $grpc.ClientMethod<$0.Empty, $0.TaskList>(
      '/theseus.Theseus/GetTaskList',
      ($0.Empty value) => value.writeToBuffer(),
      ($core.List<$core.int> value) => $0.TaskList.fromBuffer(value));
  static final _$getTaskProgress = $grpc.ClientMethod<$0.Task, $0.TaskProgress>(
      '/theseus.Theseus/GetTaskProgress',
      ($0.Task value) => value.writeToBuffer(),
      ($core.List<$core.int> value) => $0.TaskProgress.fromBuffer(value));
  static final _$streamTaskProgress =
      $grpc.ClientMethod<$0.Task, $0.TaskProgress>(
          '/theseus.Theseus/StreamTaskProgress',
          ($0.Task value) => value.writeToBuffer(),
          ($core.List<$core.int> value) => $0.TaskProgress.fromBuffer(value));

  TheseusClient($grpc.ClientChannel channel,
      {$grpc.CallOptions? options,
      $core.Iterable<$grpc.ClientInterceptor>? interceptors})
      : super(channel, options: options, interceptors: interceptors);

  $grpc.ResponseFuture<$0.Catalogue> getCatalogue($0.Empty request,
      {$grpc.CallOptions? options}) {
    return $createUnaryCall(_$getCatalogue, request, options: options);
  }

  $grpc.ResponseFuture<$0.Task> launch($0.LaunchOptions request,
      {$grpc.CallOptions? options}) {
    return $createUnaryCall(_$launch, request, options: options);
  }

  $grpc.ResponseFuture<$0.TaskList> getTaskList($0.Empty request,
      {$grpc.CallOptions? options}) {
    return $createUnaryCall(_$getTaskList, request, options: options);
  }

  $grpc.ResponseFuture<$0.TaskProgress> getTaskProgress($0.Task request,
      {$grpc.CallOptions? options}) {
    return $createUnaryCall(_$getTaskProgress, request, options: options);
  }

  $grpc.ResponseStream<$0.TaskProgress> streamTaskProgress($0.Task request,
      {$grpc.CallOptions? options}) {
    return $createStreamingCall(
        _$streamTaskProgress, $async.Stream.fromIterable([request]),
        options: options);
  }
}

abstract class TheseusServiceBase extends $grpc.Service {
  $core.String get $name => 'theseus.Theseus';

  TheseusServiceBase() {
    $addMethod($grpc.ServiceMethod<$0.Empty, $0.Catalogue>(
        'GetCatalogue',
        getCatalogue_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.Empty.fromBuffer(value),
        ($0.Catalogue value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.LaunchOptions, $0.Task>(
        'Launch',
        launch_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.LaunchOptions.fromBuffer(value),
        ($0.Task value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.Empty, $0.TaskList>(
        'GetTaskList',
        getTaskList_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.Empty.fromBuffer(value),
        ($0.TaskList value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.Task, $0.TaskProgress>(
        'GetTaskProgress',
        getTaskProgress_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.Task.fromBuffer(value),
        ($0.TaskProgress value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.Task, $0.TaskProgress>(
        'StreamTaskProgress',
        streamTaskProgress_Pre,
        false,
        true,
        ($core.List<$core.int> value) => $0.Task.fromBuffer(value),
        ($0.TaskProgress value) => value.writeToBuffer()));
  }

  $async.Future<$0.Catalogue> getCatalogue_Pre(
      $grpc.ServiceCall call, $async.Future<$0.Empty> request) async {
    return getCatalogue(call, await request);
  }

  $async.Future<$0.Task> launch_Pre(
      $grpc.ServiceCall call, $async.Future<$0.LaunchOptions> request) async {
    return launch(call, await request);
  }

  $async.Future<$0.TaskList> getTaskList_Pre(
      $grpc.ServiceCall call, $async.Future<$0.Empty> request) async {
    return getTaskList(call, await request);
  }

  $async.Future<$0.TaskProgress> getTaskProgress_Pre(
      $grpc.ServiceCall call, $async.Future<$0.Task> request) async {
    return getTaskProgress(call, await request);
  }

  $async.Stream<$0.TaskProgress> streamTaskProgress_Pre(
      $grpc.ServiceCall call, $async.Future<$0.Task> request) async* {
    yield* streamTaskProgress(call, await request);
  }

  $async.Future<$0.Catalogue> getCatalogue(
      $grpc.ServiceCall call, $0.Empty request);
  $async.Future<$0.Task> launch(
      $grpc.ServiceCall call, $0.LaunchOptions request);
  $async.Future<$0.TaskList> getTaskList(
      $grpc.ServiceCall call, $0.Empty request);
  $async.Future<$0.TaskProgress> getTaskProgress(
      $grpc.ServiceCall call, $0.Task request);
  $async.Stream<$0.TaskProgress> streamTaskProgress(
      $grpc.ServiceCall call, $0.Task request);
}
