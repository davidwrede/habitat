// Copyright:: Copyright (c) 2015-2016 The Habitat Maintainers
//
// The terms of the Evaluation Agreement (Habitat) between Chef Software Inc.
// and the party accessing this file ("Licensee") apply to Licensee's use of
// the Software until such time that the Software is made available under an
// open source license such as the Apache 2.0 License.

import "reflect-metadata";

// Expose chai.expect globally for tests
declare var expect: Function;
expect = chai.expect;

// Ensure config global exists
window["Habitat"] = { config: {} };

// Load all tests
let testContext = (<{ context?: Function }>require).context("./", true, /\.test\.ts/);
testContext.keys().forEach(testContext);
