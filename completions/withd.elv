
use builtin;
use str;

set edit:completion:arg-completer[withd] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'withd'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'withd'= {
            cand -c 'Create the directory if it does not exist.'
            cand --create 'Create the directory if it does not exist.'
            cand -t 'Create a temporary directory within the directory specified by -c/--create.'
            cand --temporary 'Create a temporary directory within the directory specified by -c/--create.'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
            cand -V 'Print version'
            cand --version 'Print version'
        }
    ]
    $completions[$command]
}
