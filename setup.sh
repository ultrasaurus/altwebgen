#!/bin/sh
# install all dependencies and set up environment
echo "setup conda with python version for whisper"
conda create -y --name whisperx python=3.10

echo "install whisperx dependencies"
conda install -y --name whisperx \
    pytorch==2.0.0 torchaudio==2.0.0 pytorch-cuda=11.8 -c pytorch -c nvidia

conda activate whisperx
conda info

echo "install whisperx"
pip install git+https://github.com/m-bain/whisperx.git

which whisperx