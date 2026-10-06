create table if not exists public.gravipop_scores (
    user_id uuid primary key references auth.users(id) on delete cascade,
    display_name text not null check (char_length(display_name) between 3 and 20),
    high_score bigint not null default 0 check (high_score between 0 and 1000000000),
    updated_at timestamptz not null default now(),
    constraint gravipop_display_name_chars check (display_name ~ '^[A-Za-z0-9 _.-]+$')
);

alter table public.gravipop_scores enable row level security;
revoke all on public.gravipop_scores from anon, authenticated;

create or replace function public.submit_gravipop_score(p_display_name text, p_score bigint)
returns void language plpgsql security definer set search_path = '' as $$
declare uid uuid := auth.uid();
begin
    if uid is null then raise exception 'Sign-in required'; end if;
    if p_score < 0 or p_score > 1000000000 then raise exception 'Invalid score'; end if;
    if p_display_name !~ '^[A-Za-z0-9 _.-]{3,20}$' then raise exception 'Name must be 3-20 letters, numbers, spaces, dots, dashes or underscores'; end if;
    insert into public.gravipop_scores(user_id, display_name, high_score)
    values (uid, p_display_name, p_score)
    on conflict (user_id) do update
      set display_name = excluded.display_name,
          high_score = greatest(public.gravipop_scores.high_score, excluded.high_score),
          updated_at = now();
end;
$$;

create or replace function public.get_gravipop_leaderboard(p_limit integer default 20)
returns table(display_name text, high_score bigint)
language sql stable security definer set search_path = '' as $$
    select s.display_name, s.high_score
    from public.gravipop_scores s
    order by s.high_score desc, s.updated_at asc
    limit greatest(1, least(coalesce(p_limit, 20), 50));
$$;

revoke all on function public.submit_gravipop_score(text, bigint) from public, anon;
grant execute on function public.submit_gravipop_score(text, bigint) to authenticated;
revoke all on function public.get_gravipop_leaderboard(integer) from public;
grant execute on function public.get_gravipop_leaderboard(integer) to anon, authenticated;
