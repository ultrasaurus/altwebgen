#!/bin/sh
# install all dependencies and set up environment
echo "setup conda with python version for whisper"
conda create -y --name whisperx python=3.10

echo "install whisperx dependencies"
conda install -y --name whisperx \
    pytorch==2.5.0 torchaudio==2.5.0 pytorch-cuda=11.8 -c pytorch -c nvidia

conda activate whisperx

echo "install whisperx"
pip install git+https://github.com/m-bain/whisperx.git

# prolly there's a better way to do this by modifying above commands
# https://github.com/Vaibhavs10/insanely-fast-whisper/issues/233
pip uninstall -y numpy
pip install numpy==1.26.3

which whisperx

whisperx --help
