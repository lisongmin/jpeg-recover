上次把照片存 u 盘的时候，没等保存完成，就把 u 盘拔了，导致 ntfs 文件系统被破坏，数据原先的照片也全没了。考虑到 u 盘没有使用多少次，存储的数据应该是连续的，于是想通过全盘搜索匹配出 jpeg 的图片，来还原照片。

编译程序

```bash
cargo build --releas
```

获取 u 盘对应分区的大小

```
$ cat /sys/class/block/sdc2/size
60305408
$ expr 60305408 \* 512
30876368896
```

调用程序搜索还原照片

```
$ ./target/release/jpeg-recover -i /dev/sdc2 -s 30876368896
```

还原的照片会保存在/tmp/recover_jpeg 目录下，注意这个命令并不能保证100%还原照片，
可以通过修改src/main.rs中的min_jpeg_size和max_jpeg_size来更精确的匹配照片。

最后将照片改名为拍照时的时间

```
cp -a ./change_file_with_time.sh /tmp/recover_jpeg
cd /tmp/recover_jpeg
./change_file_with_time.sh
```
