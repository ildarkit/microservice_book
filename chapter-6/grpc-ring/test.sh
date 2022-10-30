FIRST=127.0.0.1:4444
SECOND=127.0.0.1:5555
THIRD=127.0.0.1:6666

start_service() {
    NEXT_ADDR="http://$2"
    echo "start server $1"
    RUST_LOG=grpc_ring=trace RUST_BACKTRACE=1 ADDRESS=$1 NEXT=$NEXT_ADDR target/debug/grpc-ring > $3 2>&1 &
}

cargo build

start_service $FIRST $SECOND first.log
start_service $SECOND $THIRD second.log
start_service $THIRD $FIRST third.log

sleep 1

NEXT="http://$FIRST" target/debug/grpc-ring-client

sleep 1

pkill grpc-ring

echo FIRST
cat first.log
echo SECOND
cat second.log
echo THIRD
cat third.log

rm first.log second.log third.log
