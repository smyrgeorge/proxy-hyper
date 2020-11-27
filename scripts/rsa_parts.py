#!/usr/bin/env python3

# Scipt found here:
# https://stackoverflow.com/questions/48025710/obtain-rsa-exponent-and-modulus-from-public-key-with-python
# You may need to install pycrypto.
# pip install pycrypto

from Crypto.PublicKey import RSA
from Crypto.Util.number import long_to_bytes

import base64
import struct

key_encoded='''-----BEGIN PUBLIC KEY-----
PLACE_YOUR_PUBLIC_HERE
eg: MIG...AQAB
-----END PUBLIC KEY-----'''


pubkey = RSA.importKey(key_encoded)
print('modulus (n): {}'.format(pubkey.n))
print('exponent (e): {}'.format(pubkey.e))

