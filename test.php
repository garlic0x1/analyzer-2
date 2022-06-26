<?php
$input = $_GET[0];
if (true) {
query(intval($input));
}
$filtered = "abcdefg" . unknown(htmlspecialchars($input));

function test() {
	query(intval(htmlspecialchars($_GET[0])));
}

query(intval($filtered));
$dead = $filtered;

test();
?>
