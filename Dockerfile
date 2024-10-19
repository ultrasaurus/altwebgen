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
RUN conda install -y --name whisperx \
    pytorch==2.0.0 torchaudio==2.0.0 pytorch-cuda=11.8 -c pytorch -c nvidia

RUN echo "source activate whisperx" > ~/.bashrc
RUN source ~/.bashrc
ENV PATH=/opt/conda/envs/whisperx/bin:$PATH

ENV PIP_ROOT_USER_ACTION=ignore
RUN pip install git+https://github.com/m-bain/whisperx.git



#----- install rust and cargo ------------------------
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh  -s -- -y

WORKDIR /app
COPY . .
RUN source ~/.bashrc && cargo build
