## Experimental Web Development Server

![Tests](https://github.com/ultrasaurus/altwebgen/actions/workflows/action-test.yml/badge.svg
)

Supports
* handlebar templates (.hbs)
* directly serves all other files
* live reload: watches file system every second, reloads page on changes

Run with debug logging on:
```
cd samples/basic
RUST_LOG=debug cargo run
```

NOTE: everything will change, don't depend on this staying as is

BEWARE:
* live reload only works when JS code is manually included (see sidebar/templates/layout.hbs)
* to use automatic transcript generations (and run the tests)...
```ln -s $PWD/transcript-converter ~/transcript-converter```

# SETUP for Development

## Installing Whisper

using [miniconda](https://docs.anaconda.com/miniconda/)...


following [whisperx setup guide](https://github.com/m-bain/whisperX/blob/main/README.md#setup-%EF%B8%8F)

steps for Mac below (for linux, see `setup.sh` used by Dockerfile and github actions)
```
conda create --name whisperx python=3.10
conda install --name whisperx \
    pytorch==2.0.0 torchvision==0.15.0 torchaudio==2.0.0 \
    -c pytorch -c nvidia
conda activate whisperx
pip install git+https://github.com/m-bain/whisperx.git
````

### Error: could not run whisperx at all,

even `whisperx --help` fails

this worked (via https://github.com/pytorch/audio/issues/1573)
```
pip install -U torch torchaudio --no-cache-dir
````
> `Successfully installed torchaudio-2.5`

### compute type error (on mac)

*"ValueError: Requested float16 compute type,
but the target device or backend do not support efficient float16 computation."*

```
whisperx data/sample01.wav   --compute_type float32
```


## Building with Docker

To simulate github actions environment:
```
docker build --platform=linux/amd64  -t altwebgen-amd64 .
docker run --platform=linux/amd64 -it altwebgen
```

### Errors

Using Docker, build initially failed with "no space left on device" error. To resolve this:
```
docker system prune
docker system prune --volumes
```
Then I increased Virtual disk limit using Docker Desktop UI (Mine was 64MB I increased to 104 GB, though may have worked with less).




---

## Code Credits

* Thank you mdBook! sidebar sample theme started with mdBook theme, somewhat adjusted to fit into this format

## Image Credits
[huntsman-spider.jpeg](https://commons.wikimedia.org/wiki/File:Huntsman_spider_white_bg03.jpg) by "Fir0002/Flagstaffotos" License CC BY-NC via https://simple.wikipedia.org/wiki/Huntsman_spider

[spider-icon.webp](https://uxwing.com/spider-icon/) via https://uxwing.com/spider-icon/

[corner-cobwebs.png](https://pixabay.com/vectors/spider-web-corner-wall-design-311050/) CC0
