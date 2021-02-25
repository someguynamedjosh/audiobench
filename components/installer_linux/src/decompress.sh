#!/usr/bin/env sh
# https://www.linuxjournal.com/node/1005818

export TMPDIR=`mktemp -d /tmp/selfextract.XXXXXX`
echo "Extracting installation files to $TMPDIR..."

ARCHIVE=`awk '/^__ARCHIVE_BELOW__/ {print NR + 1; exit 0; }' $0`
echo $ARCHIVE

tail -n+$ARCHIVE $0 | tar xzv -C $TMPDIR

CDIR=`pwd`
cd $TMPDIR
./install.sh

cd $CDIR
rm -rf $TMPDIR

exit 0

__ARCHIVE_BELOW__
