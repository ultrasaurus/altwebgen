#!/bin/sh
# install all dependencies and set up environment
echo "setup conda with python version for whisper"
conda create --name whisperx python=3.10

conda activate whisperx
python --version
conda info

echo "install whisperx dependencies"
conda install pytorch==2.0.1 torchvision==0.15.2 torchaudio==2.0.2 cpuonly -c pytorch

echo "install whisperx"
pip install git+https://github.com/m-bain/whisperx.git

which whisperx
