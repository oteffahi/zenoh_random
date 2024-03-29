#!/bin/bash

# The objective of scenario1 is to observe how client and peer mode behave in a zenoh network
# This scenario deploys a zenoh network with the following structure:
#       <pub1> - <sub1> - <sub-client> - <sub2> - <pub2>
# Observations are performed by querying the network through the different nodes,
# using a client instance which is deployed within the network.

# TODO: check containers and network before deployment

docker network create zenohrand-s1-network

docker run -d --rm --name zenohrand-s1-sub1 --network zenohrand-s1-network --network-alias sub1 zenoh-random:0.1.0 sub_callback -l tcp/0.0.0.0:7337 -e tcp/pub1:7337 tcp/sub-client:7337 --no-multicast-scouting
docker run -d --rm --name zenohrand-s1-sub2 --network zenohrand-s1-network --network-alias sub2 zenoh-random:0.1.0 sub_callback -l tcp/0.0.0.0:7337 -e tcp/pub2:7337 tcp/sub-client:7337 --no-multicast-scouting
docker run -d --rm --name zenohrand-s1-sub-client --network zenohrand-s1-network --network-alias sub-client zenoh-random:0.1.0 sub_callback -m client -l tcp/0.0.0.0:7337 -e tcp/sub1:7337 tcp/sub2:7337 --no-multicast-scouting
docker run -d --rm --name zenohrand-s1-pub1 --network zenohrand-s1-network --network-alias pub1 zenoh-random:0.1.0 publisher -l tcp/0.0.0.0:7337 -e tcp/sub1:7337 --no-multicast-scouting
docker run -d --rm --name zenohrand-s1-pub2 --network zenohrand-s1-network --network-alias pub2 zenoh-random:0.1.0 publisher -l tcp/0.0.0.0:7337 -e tcp/sub2:7337 --no-multicast-scouting

echo ""
echo "Scenario1 has been deployed"
echo "In a new terminal, run the following command"
echo "docker run --rm -it --name zenohrand-s1-client --network zenohrand-s1-network --network-alias client zenoh-random:0.1.0 bash"

echo ""
echo "Phase1:"
echo "Within the client container, use the 'client' binary to query the queryables from different peers (pub1, sub1, sub-client, sub2, pub2)"
echo "Example: client -m client -e tcp/sub2:7337 --no-multicast-scouting"
echo "Observe that sub-client is only connected to sub1, and is effectively splitting the network in half"
echo "Input anything to proceed to the next phase..."
read

echo "Stopping sub1..."
docker stop zenohrand-s1-sub1

echo ""
echo "Phase2:"
echo "Sub1 has been stopped. Query the network using the client"
echo "Observe that sub-client is now listening to pub2 through the connection to sub2"

echo "Input anything to remove scenario1 instances..."
read

echo "Cleaning scenario1..."
docker stop zenohrand-s1-sub2 &
docker stop zenohrand-s1-sub-client &
docker stop zenohrand-s1-pub1 &
docker stop zenohrand-s1-client &
docker stop zenohrand-s1-pub2
docker network remove zenohrand-s1-network