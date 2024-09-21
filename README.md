# AirBnb listing similarity

### Model 

Model used is an embedding model, trained on the AirBnb dataset. Since every listing has multiple feature including raw text, some categorical features and numerical features as well, the solution that had been chosen - is to encode all features into embedding vector for listings, and do the same for review related to the listing. Then we can use triplet loss to train similarity between listings and reviews. For negative samples we can use random listings and reviews. 

The notebook is available here [Colab](https://colab.research.google.com/drive/1NmcR_cmTx1KHrqqMPvR76MhXPXc1y-tz?usp=sharing) and it takes around 4 hours to train the model on the GPU (A100) for ~10 epochs, using sample of 100k of positive/negative pairs.

### Inference

Inference is done on the SQLite side, where efficient kNN search is performed. In the notebook we create a SQLite database with the embeddings and export it to the server side. In practice we would need to update database in near-time when listings are added or updated, but for this example we just use the pre-trained embeddings. Also, in practice the most impactful signal would be the CTR on the similar candidates and optimization of such signal would be crucial.

Finding the closest embedding is done in the sqlite extension which based on [Faiss](https://faiss.ai/) library, which implements the [paper](https://arxiv.org/abs/1603.09320)

### Server Implementation

The server is implemented in Rust, and it's a simple HTTP server that accepts the POST request with the JSON payload that contains the listing_id and returns the similar listing_ids. The server is using SQLite to perform the kNN search.

### Running the server

To run the server you need to have Rust installed, then you can run the following commands:

```bash
# install rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# test the installation
rustc --version

# get the database via LFS
git lfs pull

# run the server from the root directory
cargo run --release -- database/listings.db

# to get API documentation go /swagger-ui
# better use Chrome (didn't work in Safari)
open http://localhost:9090/swagger-ui/ 
```
