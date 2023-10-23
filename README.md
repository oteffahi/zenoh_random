# Zenoh_Random

This project uses [Zenoh](https://github.com/eclipse-zenoh/zenoh) to implement a pub/sub/query system for publishing a random integer, subscribing to the published data, and querying the average of all published values through a queryable.

Note: without the use of a storage backend, the queryable has to be exposed within the same zenoh application as the subscriber. See `sub_stream.rs` and `sub_callback.rs` for more details.

## 1. Compiling

Run the following command at the root of the project:
```console
$ cargo build --release
```
The binaries `client`, `publisher`, `sub_callback` and `sub_stream` will be available in the `target/release` directory.

Alternatively, you can directly run binaries using cargo:
```console
$ cargo run --bin [binary_name] -- [parameters]
```

## 2. Usage

All examples below are running on the same host. Peers are automatically detected through multicast scouting.

### 2.1. Running a publisher

```console
$ publisher -d 5000
Opening session...
Declaring Publisher on 'test/random'...
Running with delay=5000ms
Putting Data ('test/random': '-1438684428')...
Putting Data ('test/random': '-612383115')...
Putting Data ('test/random': '1608506813')...
...
```

### 2.2. Running a subscriber

`sub_stream` and `sub_callback` are two different approaches to implementing the subscriber and queryable. They are interchangeable, and they both expose a queryable.

```console
$ sub_stream
Opening session...
Declaring Subscriber on 'test/random'...
Enter 'q' to quit...
>> [Subscriber] Received PUT ('test/random': '-1438684428')
>> [Subscriber] Received PUT ('test/random': '-612383115')
>> [Subscriber] Received PUT ('test/random': '1608506813')
...
```

### 2.3. Querying

In this example, one answer is received from the only queryable instance that is running. Consolidation is hardcoded to `None` (see FAQ for more details).

```console
$ client
Opening session...
Sending Query 'test/average'...
>> Received ('test/average': '-147520243.33333334 3')
Selected average: -147520243.33333334 with nb_values=3
```

## 3. Deploying with docker

Using the provided Dockerfile, an image containig all 4 binaires can be built with the following command:

```console
$ docker build -t zenoh-random:0.1.0 .
```

Some examples are provided in the `scenarios` directory. Make sure to build the image with the appropriate tag (`zenoh-random:0.1.0`) before running these scenarios.

## 4. FAQ

### 4.1. Difference between peer-mode and client-mode

Peers are intended to participate in the routing: they publish, subscribe, expose queryables, and can query. They connect to multiple nodes (peers, clients, routers), and forward data to them if necessary.

Clients on the other hand do not participate to the routing: they can publish, subscribe, query, and can technically expose queryables too. However, they can only have one active connection at any time, and as such, they do not forward data to other nodes. They are a leaf in the network topology.

Finally, both clients and peers can use multicast scouting.

`Scenario1` was created in order to confirm these behaviors. In this scenario, multicast scouting is disabled in order to keep the network static. `pub1`, `sub1`, `sub2` and `pub2` are statically configured (in this order) in a linear network topology, with `sub-client`, a subscriber running in client mode, positionned in the middle of the network.  Please look at `scenarios/scenario1.sh` and try running it to better understand the following explanation.

- The following error occurs when a client attempts to connect to another client.
```console
root@af587b6b010b:/# client -m client -e tcp/sub-client:7337 --no-multicast-scouting
Opening session...
[2023-10-23T11:27:55Z ERROR zenoh::net::runtime::orchestrator] Unable to connect to any of [tcp/sub-client:7337]!  at /usr/local/cargo/git/checkouts/zenoh-cc237f2570fab813/c72fdc7/zenoh/src/net/runtime/orchestrator.rs:104.
thread 'main' panicked at src/bin/client.rs:18:51:
called `Result::unwrap()` on an `Err` value: Unable to connect to any of [tcp/sub-client:7337]!  at /usr/local/cargo/git/checkouts/zenoh-cc237f2570fab813/c72fdc7/zenoh/src/net/runtime/orchestrator.rs:104.
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

- Querying `sub1` and `sub2` shows that sub-client is only connected to one side of the network, and splits the network in two.

```console
root@af587b6b010b:/# client -m client -e tcp/sub1:7337 --no-multicast-scouting
Opening session...
Sending Query 'test/average'...
>> Received ('test/average': '-73732220.1891892 37')
>> Received ('test/average': '-73732220.1891892 37')
Selected average: -73732220.1891892 with nb_values=37
root@af587b6b010b:/#
root@af587b6b010b:/# client -m client -e tcp/sub2:7337 --no-multicast-scouting
Opening session...
Sending Query 'test/average'...
>> Received ('test/average': '38977547.487804875 41')
Selected average: 38977547.487804875 with nb_values=41
```

- Stopping the `sub1` container cuts out `sub-client` from `pub1`, and forces it to try to connect to `sub2`, which puts it on `pub2`'s side of the network. This is shown by the reception of two different responses with the same counter value when querying `sub2`.

```console
root@af587b6b010b:/# client -m client -e tcp/sub2:7337 --no-multicast-scouting
Opening session...
Sending Query 'test/average'...
>> Received ('test/average': '88286110.07207207 111')
>> Received ('test/average': '228808550.02702704 111')
Selected average: 88286110.07207207 with nb_values=111
```

### 4.2. Effects of high throughput

At very low throughput, and with a very limited number of publishers and subscribers, we can almost guarantee that the response from querying a queryable will take into consideration all previously published values in the network.

However, at high throughput, and depending on the network topology and latency, it is almost certain that whenever a queryable is queried, the average within the response will not account for the publish events that are still transiting through the network. This will be explored in more detail in the scalability section below.

### 4.3. Scalability and network topology

Going from one publisher and one subscriber (and its queryable) to multiple instances of each further increases the effect of throughput on the network.

Zenoh guarantees eventual consistency: in this case, it means all subscribers will eventually receive all published values. However, at any moment in time if a snapshot of the network is taken, there will be published values that would still be transiting through the network and each subscriber would have seen a subset of all values that have ever been published, depending on said subscriber's position in the network, the network topology, and the throughput of each publisher. Querying a queryable is similar to taking a snapshot of all subscribers in the network, and thus this behavior will be reflected in the responses to the query. 

To this effect, the `client.rs` implementation for querying the queryables disables consolidation to receive all responses. Furthermore, a counter is included in the response to enable the client to select the average that takes into account the highest number of published values. The implementation uses JSON serialization for the response message.

To further demonstrate this behavior, `scenario2` is proposed. Similarly to `scenario1`, a static network is configured using multiple publishers and subscribers in peer mode, with multicast-scouting disabled. Please look at the `scenarios/scenario2.sh` topology to better understand the following results.

Querying through any peer yields 4 replies by the 4 queryable instances. A similar result can always be observed, where two values are equal, and the remaining two are strictly inferior. 

```console
root@b26057fbf082:/# client -m client -e tcp/sub2:7337 --no-multicast-scouting
Opening session...
Sending Query 'test/average'...
>> Received ('test/average': '-1628945.6721014492 552')
>> Received ('test/average': '10383097.702846976 562')
>> Received ('test/average': '10383097.702846976 562')
>> Received ('test/average': '4640913.126415094 530')
root@b26057fbf082:/#
root@b26057fbf082:/# client -m client -e tcp/pub1:7337 --no-multicast-scouting
Opening session...
Sending Query 'test/average'...
>> Received ('test/average': '32902752.212817412 827')
>> Received ('test/average': '35606743.92549476 859')
>> Received ('test/average': '35606743.92549476 859')
>> Received ('test/average': '28093891.769140165 849')
Selected average: 35606743.92549476 with nb_values=859
root@b26057fbf082:/#
root@b26057fbf082:/# client -m client -e tcp/sub4:7337 --no-multicast-scouting
Opening session...
Sending Query 'test/average'...
>> Received ('test/average': '10863074.221801665 2642')
>> Received ('test/average': '9512530.077327328 2664')
>> Received ('test/average': '11995459.628272252 2674')
>> Received ('test/average': '11995459.628272252 2674')
Selected average: 11995459.628272252 with nb_values=2674
```

By digging through the logs of each instance, we can see that the high-counter responses come from `sub1` and `sub3`, which are directly adjacent to `pub1` the publisher with the highest throughput (delay of 100ms). This configuration's responses are mainly affected by the distance to the highest throughput; however, more parameters are in play outside of a controlled environment.

## 5. On the callback-based and stream-based APIs implementation

Implementing the callback-based subscriber and queryable was a straightforward task. The stream-based implementation was much more technical.

The stream-based API allows for the subscriber and queryable to run concurrently, which means that the application can concurrently handle a publish event and respond to a query. Due to the nature of the callback functions and the concurrency, the implementation required the usage of `RwLock` for handling concurrent R/W operations, and `Arc` for thread-safety.

The following are some noteworthy behaviors:
- In the stream-based implementation, the usage of `select!` effectively linearizes the handling of publish events and queries. By adding a `sleep` call @`sub_stream:62`, we can simulate heavy computation in handling the query. The observed behavior is that publish events are not handled during the sleeping time. When the query response is sent, all publish events that were put on hold are sequentially processed. This implies the existence of a queue that can potentially overflow in high throughput environments if the query computation is heavy enough.
- In the callback-based implementation, a `sleep` call is added to the callback of the queryable @`sub_callback:67` to simulate heavy computation. This sleep call occurs after acquiring the read locks on the shared variables, and before the computation of the average using said variables. The expected behavior is that the acquired read lock would block the acquisition of the write lock by the subscriber callback. However, the observed behavior is that the write lock can still be acquired, and the values read by the queryable callback do not reflect the new updates. This behavior can probably be explained by digging into `RwLock` and `Arc`.
- In the callback-based implementation, since the callback closures can outlive the main function, values must be moved in each closure. This means that once the first callback is declared, the second callback closure cannot use the same variables since they were moved. The impromptu solution in this implementation is to clone the values before they are moved by the declaration of the first callback closure, and the usage of `Arc` guarantees that both closures would still be accessing the same data. However, this solution seems to lack scalability.