tar --format=ustar -c helloworld.txt -f os.tar
truncate -s 32M os.tar