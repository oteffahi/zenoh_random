#!/bin/bash

# The objective of scenario2 is to observe how throughput and network delay affect queryable responses
# This scenario deploys a zenoh network with the following structure, whith varying delays for publishers:
#       <sub1> - <pub1>
#                      \       
#                       <sub3> - <pub3> - <sub4>
#                      /
#       <sub2> - <pub2>
# Observations are performed by querying the network through the different nodes,
# using a client instance which is deployed within the network.

# TODO: check containers and network before deployment

docker network create zenohrand-s2-network

docker run --rm -d --name zenohrand-s2-sub1 --network zenohrand-s2-network --network-alias sub1 zenoh-random:0.1.0 sub_callback -l tcp/0.0.0.0:7337 -e tcp/pub1:7337 --no-multicast-scouting
docker run --rm -d --name zenohrand-s2-sub2 --network zenohrand-s2-network --network-alias sub2 zenoh-random:0.1.0 sub_callback -l tcp/0.0.0.0:7337 -e tcp/pub2:7337 --no-multicast-scouting
docker run --rm -d --name zenohrand-s2-sub3 --network zenohrand-s2-network --network-alias sub3 zenoh-random:0.1.0 sub_callback -l tcp/0.0.0.0:7337 -e tcp/pub1:7337 tcp/pub2:7337 tcp/pub3:7337 --no-multicast-scouting
docker run --rm -d --name zenohrand-s2-sub4 --network zenohrand-s2-network --network-alias sub4 zenoh-random:0.1.0 sub_callback -l tcp/0.0.0.0:7337 -e tcp/pub3:7337 --no-multicast-scouting
docker run --rm -d --name zenohrand-s2-pub1 --network zenohrand-s2-network --network-alias pub1 zenoh-random:0.1.0 publisher -l tcp/0.0.0.0:7337 -e tcp/sub1:7337 tcp/sub3:7337 -d 100 --no-multicast-scouting
docker run --rm -d --name zenohrand-s2-pub2 --network zenohrand-s2-network --network-alias pub2 zenoh-random:0.1.0 publisher -l tcp/0.0.0.0:7337 -e tcp/sub2:7337 tcp/sub3:7337 -d 110 --no-multicast-scouting
docker run --rm -d --name zenohrand-s2-pub3 --network zenohrand-s2-network --network-alias pub2 zenoh-random:0.1.0 publisher -l tcp/0.0.0.0:7337 -e tcp/sub3:7337 tcp/sub4:7337 -d 120 --no-multicast-scouting

echo ""
echo "Scenario2 has been deployed"
echo "In a new terminal, run the following command"
echo "docker run --rm -it --name zenohrand-s2-client --network zenohrand-s2-network --network-alias client zenoh-random:0.1.0 bash"

echo ""
echo "Within the client container, use the 'client' binary to query the queryables from different peers (pub[1-3], sub[1-4])"
echo "Example: client -m client -e tcp/sub2:7337 --no-multicast-scouting"
echo "Notice how two responses are always identical, which come from sub1 and sub3 that are directly connected to the highest throughput (pub1)"
echo "Input anything to remove scenario2 instances..."
read

echo "Cleaning scenario2..."
docker stop zenohrand-s2-sub1 &
docker stop zenohrand-s2-sub2 &
docker stop zenohrand-s2-sub3 &
docker stop zenohrand-s2-sub4 &
docker stop zenohrand-s2-pub1 &
docker stop zenohrand-s2-pub2 &
docker stop zenohrand-s2-client &
docker stop zenohrand-s2-pub3
docker network remove zenohrand-s2-network