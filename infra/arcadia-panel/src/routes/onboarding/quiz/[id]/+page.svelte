<script lang="ts">
    import Loading from '../../../../components/Loading.svelte';
    import ErrorComponent from '../../../../components/Error.svelte';
	import { page } from "$app/stores";
	import { fetchClient } from "$lib/fetch";
	import type { CreateQuizResponse } from "$lib/generated/persepolis/CreateQuizResponse";
	import OnboardingBoundary from "../../OnboardingBoundary.svelte";
	import { obBoundary } from "../../obBoundaryState";
	import { persepolisUrl } from "../../onboardingConsts";
	import InputText from '../../../../components/inputs/InputText.svelte';
	import InputTextArea from '../../../../components/inputs/InputTextArea.svelte';
	import Select from '../../../../components/inputs/select/Select.svelte';
	import { success } from '$lib/toast';
	import ButtonReact from '../../../../components/button/ButtonReact.svelte';
	import { Color } from '../../../../components/button/colors';

    const minShortAnswerLength = 50;
    const minLongAnswerLength = 750

    let quizRequest: CreateQuizResponse;
    const fetchQuiz = async () => {
        let quizDat = await fetchClient(`${persepolisUrl}/quiz`, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({
				id: $page?.params?.id || '',
				login_token: $obBoundary?.token
			})
		});

        if (!quizDat.ok) {
			let err = await quizDat.text();
			throw new Error(err?.toString() || 'An unknown error occurred while loading the quiz');
		}

        let quiz: CreateQuizResponse = await quizDat.json();

        quizRequest = quiz;

        return quiz;
    }

    const submitQuiz = async () => {
        let i = 0;

        let answers: { [key: string]: any } = {}
        for(let question of (quizRequest?.questions || [])) {
            if (question == undefined) {
                throw new Error('Internal error: question is undefined');
            }

            let resp = respData[i];

            if (!resp) {
                throw new Error(`Question ${i + 1} is not answered!`);
            }

            if (question.data == "short") {
                if (resp.length < minShortAnswerLength) {
                    throw new Error(`Question ${i + 1} is too short!`);
                }
            } else if (question.data == "long") {
                if (resp.length < minLongAnswerLength) {
                    throw new Error(`Question ${i + 1} is too short!`);
                }
            } else if (question.data.multiple_choice) {
                if (!resp) {
                    throw new Error(`Question ${i + 1} is not answered!`);
                }
            } else {
                throw new Error(`Question ${i + 1} is not answered!`);
            }

            answers[question?.question] = resp;

            i++;
        }

        let res = await fetchClient(`${persepolisUrl}/submit-quiz`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                id: $page?.params?.id || '',
                login_token: $obBoundary?.token,
                quiz_answers: answers,
                sv_code: svCode
            })
        })

        if (!res.ok) {
            let err = await res.text();
            throw new Error(err?.toString() || 'An unknown error occurred while submitting the quiz');
        }

        success('Quiz submitted successfully!');
        return true
    }

    let respData: any = {}
    let svCode: string = ''
</script>
<OnboardingBoundary>
	{#await fetchQuiz()}
		<Loading msg="Fetching guide..." />
	{:then resp}
        <details class="mb-4 text-white">
            <summary class="text-white hover:cursor-pointer">Debug</summary>
            <div class="overflow-x-scroll">
                <pre>{JSON.stringify(resp, null, '\t')}</pre>
            </div>
        </details>
        {#each resp?.questions as question, i}
            {#if question.data == "short"}
                <InputText 
                    id={`question-${i}`}
                    label={question.question}
                    placeholder={"This is a short answer question!"}
                    disabled={false}
                    required={true}
                    minlength={minShortAnswerLength}
                    bind:value={respData[i]}
                />
            {:else if question.data == "long"}
                <InputTextArea
                    id={`question-${i}`}
                    label={question.question}
                    placeholder={"This is a looooong answer question, so answer wisely!"}
                    disabled={false}
                    required={true}
                    minlength={minLongAnswerLength}
                    bind:value={respData[i]}
                />
            {:else if question.data.multiple_choice}
                <Select 
                    id={`question-${i}`}
                    label={question.question}
                    disabled={false}
                    required={true}
                    bind:value={respData[i]}
                    choices={[
                        ...question.data['multiple_choice'].map((option, i) => {
                            return {
                                id: i.toString(),
                                label: option,
                                value: option
                            }
                        })
                    ]}
                />
            {/if}
        {/each}

        <hr class="mt-10" />

        <InputText
            id={'question-svi'}
            label={'Staff Verification Code'}
            bind:value={svCode}
            placeholder={"You can find this from the staff guide if you didn't record it!"}
            disabled={false}
            required={true}
            showErrors={true}
            minlength={3}
        />

        <ButtonReact
            color={Color.Themable}
            icon={'mdi:send'}
            text={'Submit Quiz'}
            onClick={submitQuiz}
            states={{
                loading: 'Submitting...',
                success: 'Submitted!',
                error: 'Failed to submit quiz!'
            }}
        />
	{:catch error}
		<ErrorComponent msg={`Something went wrong: ${error.message}`} />
	{/await}
</OnboardingBoundary>