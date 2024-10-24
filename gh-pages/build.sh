#! /bin/sh
echo $1
PREFIX=${1:-"/"}    

echo "pwd: " `pwd`
(
cd gh-pages
echo "---> set up fresh 'content' directory"
rm -rf content
)

OUT_DIR="gh-pages/content/_website"
mkdir -p $OUT_DIR


(cd ./samples/media; webgenr -p $PREFIX/media-sample build)
echo "pwd: " `pwd`
mv ./samples/media/.dist $OUT_DIR/media-sample
echo "ls $OUT_DIR"
ls -lR $OUT_DIR


## add build info to index page
(
echo "---> add build info to index page"
cd gh-pages/content

mkdir template
cp ../default_content.hbs template/default.hbs

mkdir source
INDEX_FILE="source/index.md"
cp ../index_content.md $INDEX_FILE

DATE_STRING=$(date +"%a, %b %d %Y - %I:%M %p")
echo $DATE_STRING

cat <<EOT >> $INDEX_FILE
\`\`\`
$DATE_STRING
branch=$BRANCH_NAME
$GITHUB_SHA
\`\`\`
EOT

webgenr -p $PREFIX build
mv .dist/index.html _website/.

echo "---> files that will be deployed <---"
ls -lR _website
)


