# RCO Node
FROM julia:1.12-bookworm

WORKDIR /app
RUN mkdir -p /app/dist /app/crates/rco-sdk-julia/julia

# Copy files
COPY dist/librco.so /usr/local/lib/librco.so
COPY dist/manifest.json /app/dist/
COPY crates/rco-sdk-julia/julia /app/crates/rco-sdk-julia/julia
COPY scripts/start.jl /app/scripts/

RUN ldconfig

# Dependencies
RUN julia -e 'using Pkg; Pkg.add("JSON")'

# Start
ENTRYPOINT ["julia", "/app/scripts/start.jl"]
