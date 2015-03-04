'use strict';

var debug = require('debug')(__filename.slice(__dirname.length + 1));
var fs = require('fs');
var util = require('./crater-util');
var tc = require('taskcluster-client');
var Promise = require('promise');
var slugid = require('slugid');

var defaultCredentialsFile = "./tc-credentials.json";

function main() {
  var toolchain = util.parseToolchain(process.argv[2])
  if (!toolchain) {
    console.log("can't parse toolchain");
    process.exit(1);
  }

  debug("scheduling for toolchain %s", JSON.stringify(toolchain));

  var credentials = loadCredentials(defaultCredentialsFile);

  debug("credentials: %s", JSON.stringify(credentials));

  scheduleTasks(toolchain, credentials);
}

function loadCredentials(credentialsFile) {
  return JSON.parse(fs.readFileSync(credentialsFile, "utf8"));
}

function scheduleTasks(toolchain, credentials) {
  var queue = new tc.Queue({
    credentials: credentials
  });

  // Get the task descriptors for calling taskcluster's createTask
  var taskDescriptors = getTaskDescriptors(toolchain);
  taskDescriptors.forEach(function (task) {
    debug("createTask payload: " + JSON.stringify(task));

    var taskId = slugid.v4();

    debug("using taskId " + taskId);

    var p = queue.createTask(taskId, task);

    var p = p.catch(function (e) {
      console.log("error creating task: " + e);
      process.exit(1)
    });
    var p = p.then(function (result) {
      console.log("createTask returned status: ", result.status);
      console.log("inspector link: https://tools.taskcluster.net/task-inspector/#" + taskId);
    });
  });
}

function getTaskDescriptors(toolchain) {
  // TODO


  var channel = toolchain.channel;
  var archiveDate = toolchain.date;
  var crateName = "toml";
  var crateVers = "0.1.18";

  var deadlineInMinutes = 60;
  var rustInstallerUrl = installerUrlForToolchain(toolchain);
  var crateUrl = "https://crates.io/api/v1/crates/" + crateName + "/" + crateVers + "/download";

  var taskName = "nightly-2015-03-01-vs-toml-0.1.18";

  var createTime = new Date(Date.now());
  var deadlineTime = new Date(createTime.getTime() + deadlineInMinutes * 60000);

  // Using b2gtest because they have active works available
  var workerType = "b2gtest";

  var env = {
    "CRATER_RUST_INSTALLER": rustInstallerUrl,
    "CRATER_CRATE_FILE": crateUrl
  };
  var cmd = "apt-get update && apt-get install curl -y && (curl -sf https://raw.githubusercontent.com/brson/taskcluster-crater/master/run-crater-task.sh | sh)";

  var task = {
    "provisionerId": "aws-provisioner",
    "workerType": workerType,
    "created": createTime.toISOString(),
    "deadline": deadlineTime.toISOString(),
    "routes": [
      "crater.#"
    ],
    "payload": {
      "image": "ubuntu:13.10",
      "command": [ "/bin/bash", "-c", cmd ],
      "env": env,
      "maxRunTime": 600
    },
    "metadata": {
      "name": "Crater task " + taskName,
      "description": "Testing Rust crates for Rust language regressions",
      "owner": "banderson@mozilla.com",
      "source": "http://github.com/jhford/taskcluster-crater"
    },
    "extra": {
      "crater": {
	"channel": channel,
	"archiveDate": archiveDate,
	"crateName": crateName,
	"crateVers": crateVers
      }
    }
  };
  return [task];
}

function installerUrlForToolchain(toolchain) {
  // FIXME
  var url = "http://static-rust-lang-org.s3-us-west-1.amazonaws.com/dist/";
  var url = url + toolchain.date + "/rust-" + toolchain.channel + "-x86_64-unknown-linux-gnu.tar.gz";
  return url;
}

main();