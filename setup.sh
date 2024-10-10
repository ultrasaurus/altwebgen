#!/bin/sh
# install all dependencies and set up environment
echo "setup conda with python version for whisper"
conda create --name whisperx python=3.10

conda activate whisperx
python --version
conda info

echo "install whisperx dependencies"
conda install pytorch==2.0.0 torchaudio==2.0.0 pytorch-cuda=11.8 -c pytorch -c nvidia

echo "install whisperx"
pip install git+https://github.com/m-bain/whisperx.git

which whisperx
