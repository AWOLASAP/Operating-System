**Run this after you've removed branches from github (prevents stale branches from showing up in git branch -a)**

git remote prune origin 

**Run this to get changes, but not to merge them with your local master branch**

git fetch

**Run this to delete a branch**

git branch -d <branchname>

**Run this to list all the branches that exist**

git branch -a

**Run this to switch branches**

git checkout <branchname>

**Run this to push from a branch**

git push --set-upstream origin <branchname>

**create a branch and check it out**

git checkout -b <branchname>
