# theseus_gui
compile proto

protoc --dart_out=grpc:lib/generated --proto_path=../theseus_daemon/src/protos -Iprotos ../theseus_daemon/src/protos/theseus.proto
