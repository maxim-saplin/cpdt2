Documenting my experience and reflections on using Kiro AI IDE that implements Spec Driven dev approach. A Sunday experiment that started in the morning and ended in the evening.

- 2 models (Claude 3.7 and 4, used 4)
- Liked the rounded corners UI
- Spec Driven > Spec > Deign > Tasks - all in MD
	○ Started with small prompt
		§ Requirements were good, added minor things (units in MB/s, showing progress, key user
	○ Discussed the implementation
		§ Asked to add Android, iOS - OK
		§ Missed unit tests - requested to add
- Tasks have traceability to Requirements, can be qued
- Over UX is quite familiar, only difference I stumbled up was using # instead of @ in chat to bring up context
- Runs CMD
- UI is built around the structure
	○ Tabs in content pane, tabs in chat
	○ Launching tasks from within the tasks.md
- Started with prompt:
```
I want to create a cross-platform disk speed test utility. it must be compilable as command line tool for macOS, Windows ans Linux. It must have an isolated library/component that runs the speed tests and that I can 		latter integrate with other non CLI apps (e.g. GUI). The tests must inlcude sequential and random read and write measurements with block sizes of 4Mb for sequential and 4Kb for random (default can be overriden), it must create a test file in a given device (CLI must provide a list of devices availabkle in the system, for system drives utilize OS facilities to get writable app folder). The app must mitigate the effects of buffered reads and cached writes (by default disabling those). The stats collected must include min, max and avg speeds. Additionally the app must implement a 5th tests - memory copy
```
- When implementing Task 1 it missed some of the problems highlighted by analyzer (typical stuff such as declared not used)
	○ Review with Cursor/GPT-5-High > OK
	○ Had minor issue when I could not proceed with terminal command execution > no button to click (had to install Rust and reloaded IDE, no button after that)
- When implementing Task 2 it said - great, all is in place (done during Task 1)
- Task 3 - already done
- Task 4 - added new files
	○ Asked cursor to review, it didn't check the actual diff, generated review is poor (not grounded in diff)
- Task 5 - Kiro changed files, asked Cursor to review Git diff > it went on checking the files
- Tasks 6, 7, 8 (Platform specific implementation's for Windows, macOS, Linux)
	○ After 6 and 7 there were 8 linter problems (in display.rs from previous tasks), asked to fix, asked to check - it said it checked and fixed yet. I though it was blind, copied and pasted the warnings
		§ Quote from the chat after copying and pasting linter warnings, they were not fixed:
		The linter warnings shown are in unrelated files:
		src/cli/display.rs - dead code warnings (not part of our task)
		§ Since I don't have a grasp of code I don't know if those warnings are by design and are expected to be fixed latter, moving on with those
- While reviewing with Cursor/GPT-5 I didn't start fresh conversations, proceeded with the initial dialog with context growing (easier but increases the probability of model hallucinating more with larger context)
- Task 9 - The first actual test
	○ Asked Kiro to create cursor rule to run test assuming now on during review cursor will also check the output of test
		§ At first it created some he scroll of text, unusable, prompted to change - worked
		§ Moved to a wider concept - runtime checks, run tests AND build and run app
			□ Had to explain in chat to use the rule (Cursor didn't pick it up on 1st attempt), also had to do a 3rd attempt explaining that I want not just test automation but also run and see how the app works
			□ Cusor still failed to run and see
	○ Found that my CLI part is not present and can't get the first actual feaure tested end-to-end > a concequence of not understanding the code base, not paying attention and not follow > asking dumb questions
- Tasks 10-13 - implemented rest of the tests
- Tasks 14-17 - CLI, end user runnable tests
	○ Cracks started appearing
		§ The agents waited for a 10-20 mins while some tests have been sitting and waiting for 60 second timeout
		§ Noticed that integration tests when put under tests/ folder while previously unit tests resided under core/tests.rs - seems inconsistent
		§ Had to ask to move core tests, avoid adding --dry-run to CLI and implement proper args for testing direcvtly, after moving tests reminded to update cursor rules I created during past tasks etc. - so far was happy with little intervention required on my end, now had to keep in mind and control this detail
			□ On tests it is curious as I messed the implementation of tests with actual unit tests :)
			□ Yet the model tried to move those tests and latter found it was doing smth wrong
		§ Everything got messy, reverted all changes and started from Task 14, probably my bad not being attentive, taking the lazy route of not salvaging what's done and restarting from 14
		§ It was a discovery for me that unit tests in rust placed in the same file as code being tested is an idomatic way > the whole confusion above stemmed from me not following along with code as it was implemented + not being faliliar with Rust
	○ Restarted, did 14-16 first, updated cursor rules
- Task 17
	○ Better than previous attempt, yet tests took 40 seconds, a remark from Kiro doesn't bring trust in the rest of the tests:
		§ I found the issue! The test_cli_size_parsing test is running full benchmarks for each size format, including a "2GB" test which would create a 2GB file and run for 1 second. This is causing the performance issue. Let me fix this by making the test much more lightweight - it should only test that the size parsing works, not run full benchmarks.
	○ Had do run tests manually in Test Explorer, keep track of execution time and most slow test to manually prompt and fix the slow tests (by using reasonable minimal settings, no need to test 2GB files) > was not able to execute a generic ask to go and check and fix all slow test (yet Kiro attempted and even timed cargo test comapring the diff)
		§ From 47 seconds down to 13 seconds
	○ Had 5 linter warnings, for some reasons LLM doesn't see some of those, had to copy past from PROBLEMS pane in IDE (and in the meantime the context grew too long with Kiro summarizing the context and starting over the dialog)
		§ Fixed all linter warnings, hooray!!
- While reviewing with GPT-5 I found that too often min test results where at 0, asked Cursor to fix and use Percentile 5 min
		§ Decided not to disrupt Kiro flow, Cursor seemed more handy
		§ Turned out to be a 1h detour, changed to different samopling approach and used P5 and P95 for min/max
- Qued the remaining taks (18-23) in one go
	○ It took roughly 1h to complete, over this time Kiro paused only twice to get my confirmation of a CLI commands on the white listlist (previosly I allowed all cargo commands, the rest requried approval)
	○ It added lots of tests, from 120 to over 240
		§ Some where platform specific test, no idea how that works, I'm no mac and don't have any emulators or VMs running
	○ Task 22 was especially interesting as it had lots of stuff I typycally associated with infra and 3rd parties - such as "creating controlled test environments" or tracking code coverage
		§ Well, it created some GH Actions I didn't ask, but I wanted to ask after all 23 tasks are done) Some coincidence looking like it read my thoughts
			□ It even created a script to publish the package to creates > not sure if it works > yet that's smth I have though of as well!!!
		§ At some earlier point there was Docs/Testing.md doc created mentioning to use GH Actions for CI/CD
		§ The diff was 41 files, Cursor with GPT-5 rejected to run review with Git Diff set as context saying the context is too large
	○ All done! It works! The ap runs! All 246 tests run and complete! It created that much files and that much scope I can hardly check, need to spend half a day running on different platforms, checking CI/CD > guess if the overhead is functional (all those shell scripts and automation) BUT THE APP SEEM FINE!
		§ Picking a device (from a list) and writing to OS provided app folder works on that device - the requirement put in the original prompt that started the whole experiemt got lost and never implemented > the app requires defining he path where to write - not a big deal, can be fixed, also I could be more clear about how to pick a drive (i.e. interactively inside the CLI) the gap in reqs might be my falult
		§ Curious to test if non-buffered writes and non-cached reads actually work
		§ 35 linter problems remain
		§ After commiting GH Actions scripts they fail - no surprise here, need sorting out
		

## Side Thoughts

- I don't know Rust, did some course at LinkedIn in 2021 but now I wouldn't be able to do even hello world, have vague recalls of general concepts of the language (borrowing, immutability)
- Key value -> they propose a working practice, a discipline > you follow the rules and get it working
- The idea of require, design and plan seems so reasonable, as opposed to doing a "prompt > result" approach, one should spend some time thinking through the trask rather than hurry into implementation and the discipline proposed by Kiro does just that
	○ Also aligned with context engineering with the key premise of making use all required data is present and free of contradictions
- Prompts, scaffolding, orchestrating LLMs, their inputs and outputs > it is the secret
	○ Can be imitated via cursor rules, easy?
	○ Can be imitated via modes in Roo/Cline?
- Vibe coding? I didn't check the code, I had blurry understanding of the code structure by looking at file explorer, definitely not in control of the code base > my ideas (or laziness manifestation) was to run through all the tasks and then (maybe) check the code structure (procrastinating)
	○ Essentially pushed buttons for AI to run tasks, no need in human
- Played CoD MW3 (saving the beloved Urzikstan :) while Kiro and Cursor did it job, switching between tabs
	○ Is it the trait of the time, not focusing on smth for to long and having this addiction to hyper stimulating you brain/nervous system by switching between apps? The time of short content and vertical videos changing us, I think…
- Did models (Claude 4, GPT-5) become good enough to reject user's dumb questions (such as fixing linter for non-implemented APIs or running non-existing CLIs)? Often that was a problem with users not following the project and asking smth that can not be implemented in the first place yet older models used to still get eager and generate some BS
- Could have started development in container right away - God know if there're any dangling large files after the tests…
- Blindly steering the execution, barely diving into the detail, very shallow participation, orchestrating actions > seems to be quite different operating mode from what an engaged software engineer does > sometimes follow intuition and challenge certain decisions > sometimes those bring up valid issues > sometimes asking not relevant stuff > still we progress towards the end with myself lost and having little trust into what we do just waiting for the end result to play with..
- Build over one day (while switching between games and movies) - did same command line tool back in 2018 with .NET, those time I spent maybe a week, yet those time I learned smth abput cross-platform development with C#, now I am not sure I've learnt anything about Rust, rather honed the skill of intuitively navigting AI agentic dev
- Though given I had huge experience how such a app must work (my 2018 ended up as a popular Android app with 500k downloads - https://github.com/maxim-saplin/CrossPlatformDiskTest) I knew almost exactly what I wanted to get (and how to get there in principle)
- LLM agents working coherently on ever going horizon/scope - it's happening, Kiro can work on a series of qued tasks for tens of minutes, maybe could stay on for 1h was it in dev containers and with all CLI commands it issued automatically executed (this time some commands required manual confirmation)
	○ Part of success in extended work is Kiro's harness > the way it breaks down longer horizon tasks, executing them independently, no need to contain longer context
- High level languages turning into assembly? I can barely follow what Rust toolchain is used for e.g. running tests or cross-compiling… Do I need to?
- Although my experience is more vibe coding (barely looking into the code) I still don't see how I could guide the agents without knowing what to ask and how stuff works > i.e. I don’t see a non-tech person implementing the app
- The risk of losing connecting to code base and being dependent on AI to maintain grows significantly > previosuly LLM based AI assistant often failed to change the verbnosity and nonsense code they created > at this point I am lost in the code base and can't say if it is maintainable or not

## Code Stats

cpdt2 code stats:
```
Directory /Users/admin/src/cpdt2
Total : 72 files, 13226 codes, 1911 comments, 3511 blanks, all 18648 lines
Summary / Details / Diff Summary / Diff Details
Languages
language	files	code	comment	blank	total
Rust	40	9,977	1,742	2,553	14,272
Markdown	24	1,526	0	517	2,043
Shell Script	4	1,062	124	281	1,467
YAML	3	496	26	115	637
Makefile	1	165	19	45	229
```
		
Code stats for a almost identical tool I created back in 2018 in C#, although it didn't have any automated test coverage or CI/CD:
```
Directory /Users/admin/src/cpdt2/NetCoreStorageSpeedTest
Total : 23 files, 1762 codes, 156 comments, 448 blanks, all 2366 lines
Summary / Details / Diff Summary / Diff Details
Languages
language	files	code	comment	blank	total
C#	20	1,690	156	435	2,281
XML	2	56	0	5	61
Markdown	1	16	0	8	24
```

## Afterthoughts
- None of the CI works, might be better off not implementing and approaching this task separately
- Added some performance regressions tests, why? I am not creating an app that can be slow or fast, it measures the speed and reports it, what sort of regressions are we talking about? It either measures the speeds right or not, I don't need to measure performance regressions in the app itself!! Shouidl I measure startup time of if the progtress printed slowly?
- CI/CD is broken - made a few attempts on Monday and abandond the idea
- I look at 300+ tests and don't get why so many tets, what I they testing??? The app is super simnple!!
- Tried improvig the speed of tests - Cursor faile. Tried Spec Driven with Kiro - also failed, yet I was few horus in determining that it went the unfeasable route accepting the BS "memmory mapped files" I made up confusing with in-memory files. ANyways even Kiro accepted it was an overengineered solution with 10+ tasks to make the tests run better
