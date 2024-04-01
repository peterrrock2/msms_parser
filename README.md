# MSMS Parser

This is just a parser for the outputs of the 
multi-scale-map-sampler project (a.k.a Forest Recom)
that allows for the translation between the standard output
of the julia code to the standardized JSONL format and
the compressed BEN format.


The CLI for this can be installed directly from git using cargo
via the command

```
cargo install --git https://github.com/peterrrock2/msms_parser.git 
```

The main thing that this CLI does is enable the encoding of standard JSONL
outputs of the [multi-scale map sampler](https://github.com/peterrrock2/multi-scale-map-sampler.git)
(MSMS) in the standardized JSONL format:

```
{"assignment": <assignment-vector>, "sample": <sample number>}
```

or in the [BEN](https://github.com/peterrrock2/binary-ensamble.git) format. 

## Usage

Here are a list of the flags for the CLI

- `-g --graph-json` The path to the dual-graph json file that was used to generate the output
  of the MSMS

- `-i --input-jsonl` (Optional) The JSONL output of MSMS. If not passed, it is assumed that the 
  input is piped in from stdin.

- `-o --output-file` (Optional) The name of the output file for the parsing. It is recommended
  that if you are parsing using the standard mode (without the `--ben` flag) that you
  include ".jsonl" as the file extension, and if you are using the `--ben` flag, then
  include ".ben" as the file extension.

- `-r --region` The main region for use in the MSMS algorithm. (This can be obtained from the
  "levels in graph" key in the 2nd line of the standard output of MSMS.)

- `-s --subregion` The subregion used in the MSMS algorithm. (This can be obtained from the
  "levels in graph" key in the 2nd line of the standard output of MSMS.)

- `-b --ben` A boolean flag that, when included, indicates that the output should be written
  in the `ben` format.

- `-v --verbose` A boolean flag that, when included, will write some progress indicators to
  stderr


You can see the `msms_parser` at work by running the following command on the example file:

```
msms_parser -g 7x7.json -i 42_atlas_gamma0.0_100.jsonl -r county -s precinct
```

(this assumes that `~/.cargo/bin/` is in your path and that you have installed the package).
This will print the output to the console. If you choose to create an output file using
the `-o` flag, you will also see a "*.msms_settings" file generated. This is a convenience
feature that ensures that the file that is generated with `-o` is always associated with 
a settings file that tells the user how the original output was generated in the
event that they need to replicate the work. So, the command 

```
msms_parser -g 7x7.json -i 42_atlas_gamma0.0_100.jsonl -r county -s precinct -o test_out.jsonl
```

will produce the canonically formatted "test_out.jsonl" file and the associated settings
file "test_out.jsonl.msms_settings".
