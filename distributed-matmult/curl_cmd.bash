#!/usr/bin/env bash

curl -X POST http://localhost:8000/multiply_matrices_distributed \
     -H "Content-Type: application/json" \
     -d '{
           "left": [[1, 2, 3],
                        [4, 5, 6]],
           "right": [[ 7,  8],
                        [ 9, 10],
                        [11, 12]]
         }'