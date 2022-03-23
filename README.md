# odm_postprocess
## OpenDroneMap Post-Processing

A preliminary program for parsing the output created by OpenDroneMap's [NodeODM](https://www.opendronemap.org/nodeodm/) program in a way that can be uploaded to the geospatial data repository located at https://cmoran.xyz/geospatial. In summary, it both creates creates a more simple `summary.json` file containing pertinent information for georeferencing the ODM-created orthographic image, as well as creates a smaller web-compaible .webp file for being served on the map itself without the data overhead of the lossless .png file. Code for parsing and rendering this output is located [here](https://github.com/quietlychris/site).

```sh
$ cargo run --release $input_dir $output_dir
```

Please note that ODM orthophotos may be quite large in size; as a result, the default memory limits on the image processing code have been removed, and could result in a crash if you attempt to process a large image without having sufficient RAM. 

### License

This code is licensed under AGPLv3