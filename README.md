# ibc-grpc-server
To work with IBC relayer, e.g Hermes, properly. We must implement some required grpc query services. By far, we have implemented services required by ics02_client、ics03_connection and ics04_channel.    
This crate provides all these services by exposing the `run_ibc_grpc` function. To use `run_ibc_grpc`, you must implementation trait `IbcStore` first.   
