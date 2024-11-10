FROM ubuntu:latest

#----- install linux dev tools and conda -----
# https://fabiorosado.dev/blog/install-conda-in-docker/
# Install base utilities
RUN apt-get update \
    && apt-get install -y build-essential \
    && apt-get install -y wget \
    && apt-get install -y curl \
    && apt-get install -y git \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Install miniconda
ENV CONDA_DIR=/opt/conda
RUN wget --quiet https://repo.anaconda.com/miniconda/Miniconda3-latest-Linux-x86_64.sh -O ~/miniconda.sh && \
    /bin/bash ~/miniconda.sh -b -p /opt/conda

# Put conda in path so we can use conda activate
ENV PATH=$CONDA_DIR/bin:$PATH

#----- install whisperx ------------------------
SHELL ["/bin/bash", "-c"]

RUN conda create -y --name whisperx python=3.10

# Note: whisper instructions call for torchaudio==2.0.0
#       pytorch==2.5.0 and torchaudio==2.5.0 resolved the following error
# OSError: /opt/conda/envs/whisperx/lib/python3.10/site-packages/torchaudio/lib/libtorchaudio.so: undefined symbol: _ZN2at4_ops10zeros_like4callERKNS_6TensorEN3c108optionalINS5_10ScalarTypeEEENS6_INS5_6LayoutEEENS6_INS5_6DeviceEEENS6_IbEENS6_INS5_12MemoryFormatEEE
RUN conda install -y --name whisperx \
    pytorch==2.5.0 torchaudio==2.5.0 pytorch-cuda=11.8 -c pytorch -c nvidia

RUN echo "source activate whisperx" > ~/.bashrc && source ~/.bashrc
    # pip install -U torch torchaudio --no-cache-dir

ENV PATH=/opt/conda/envs/whisperx/bin:$PATH

ENV PIP_ROOT_USER_ACTION=ignore
RUN pip install git+https://github.com/m-bain/whisperx.git

# prolly there's a better way to do this by modifying above commands
# https://github.com/Vaibhavs10/insanely-fast-whisper/issues/233
RUN pip uninstall -y numpy
RUN pip install numpy==1.26.3


# ----- install rust and cargo ------------------------
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh  -s -- -y

WORKDIR /app
COPY . .
RUN source ~/.bashrc && cargo build
