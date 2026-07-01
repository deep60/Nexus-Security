rule PHP_Webshell_Indicators : suspicious
{
    meta:
        author      = "Verdyx"
        description = "Flags PHP files that combine inbound request data with dynamic code execution (common webshell pattern)"
        severity    = "medium"

    strings:
        $php   = "<?php"
        $eval  = "eval("
        $assert = "assert("
        $system = "system("
        $shell = "shell_exec("
        $passthru = "passthru("
        $req_get  = "$_GET"
        $req_post = "$_POST"
        $req_req  = "$_REQUEST"

    condition:
        // A PHP file that takes request input AND has a dynamic-execution sink.
        $php
        and any of ($req_get, $req_post, $req_req)
        and any of ($eval, $assert, $system, $shell, $passthru)
}
