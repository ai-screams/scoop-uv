Can I use scoop with conda environments?

Not directly. scoop-uv is designed to be a lightweight, fast alternative to the heavy footprint of Anaconda/Conda. It uses uv under the hood to manage virtual environments. While you can have both installed on your system, they operate differently:

Conda manages its own binaries and non-Python dependencies.

scoop-uv leverages your existing Python installations to create standard, fast virtual environments. If you are doing heavy data science requiring non-Python libraries (like MKL), Conda is great. For almost everything else, scoop-uv will be significantly faster and more portable.
