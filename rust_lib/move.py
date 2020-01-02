import shutil
import sys
import os
from subprocess import call

r=call(['cargo', 'build', '--release'])

file = sys.argv[1]
if r ==0:
    file = sys.argv[2]
    if os.path.isfile(file):
        os.remove(file)
    shutil.copy("rust_lib/target/release/sc2_helper.dll", file)
    print("File Moved")