import shutil
import sys
import os
from subprocess import call
print(sys.argv)
mode = sys.argv[2]
print(sys.argv[2])


if mode =="release":
    r=call(['cargo', 'build', '--release'])
else:
    r=call(['cargo', 'build'])

file = sys.argv[1]

if r ==0:
    file = sys.argv[1]
    # if os.path.isfile(file):
    #     os.remove(file)
    if mode =="release":
        shutil.copy("target/release/sc2_helper.dll", file)
    else:
        pass
        shutil.copy("target/debug/sc2_helper.dll", file)
    print("File Moved")